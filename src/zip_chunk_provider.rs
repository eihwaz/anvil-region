use crate::{AnvilChunkProvider, AnvilRegion, ChunkLoadError, ChunkSaveError, RegionAndOffset};
use nbt::CompoundTag;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

pub use zip::result::ZipError;

/// The chunks are read from a zip file
#[derive(Debug)]
pub struct ZipChunkProvider<R: Read + Seek> {
    zip_archive: ZipArchive<R>,
    // Prefix for the region folder. Must end with "/".
    // For example: "region/", "world/region/" or "saves/world/region/"
    region_prefix: String,
    // Cache (region_x, region_z) to uncompressed file, so each region file is
    // only uncompressed once
    cache: HashMap<(i32, i32), Vec<u8>>,
}

#[derive(Debug)]
pub enum ZipProviderError {
    Io(io::Error),
    Zip(ZipError),
    RegionFolderNotFound,
    MoreThanOneRegionFolder,
}

impl From<io::Error> for ZipProviderError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ZipError> for ZipProviderError {
    fn from(e: ZipError) -> Self {
        Self::Zip(e)
    }
}

// Find the path of the region folder inside the zip archive.
// For example: "region/", "world/region/" or "saves/world/region/"
// Panics if no region folder is found
// Panics if more than one folder is found
fn find_region_folder_path<R: Read + Seek>(
    zip_archive: &mut ZipArchive<R>,
) -> Result<String, ZipProviderError> {
    let mut region_prefix = String::from("/");
    let mut found_region_count = 0;
    for i in 0..zip_archive.len() {
        // This unwrap is safe because we are iterating from 0 to len
        let file = zip_archive.by_index(i).unwrap();
        let full_path = file.sanitized_name();
        // file_name() returns None when the path ends with "/.."
        // we handle that case as an empty string
        let folder_name = full_path.file_name().unwrap_or_default();
        if folder_name == "region" {
            found_region_count += 1;
            region_prefix = file.name().to_string();
            // Keep searching after finding the first folder, to make sure
            // there is only one region/ folder
        }
    }
    if found_region_count == 0 {
        return Err(ZipProviderError::RegionFolderNotFound);
    }
    if found_region_count > 1 {
        return Err(ZipProviderError::MoreThanOneRegionFolder);
    }

    Ok(region_prefix)
}

impl<R: Read + Seek> ZipChunkProvider<R> {
    pub fn new(reader: R) -> Result<Self, ZipProviderError> {
        let mut zip_archive = ZipArchive::new(reader)?;
        let region_prefix = find_region_folder_path(&mut zip_archive)?;
        let cache = HashMap::new();

        Ok(ZipChunkProvider {
            zip_archive,
            region_prefix,
            cache,
        })
    }

    fn region_path(&self, region_x: i32, region_z: i32) -> String {
        format!("{}r.{}.{}.mca", self.region_prefix, region_x, region_z)
    }

    pub fn load_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Result<CompoundTag, ChunkLoadError> {
        let RegionAndOffset {
            region_x,
            region_z,
            region_chunk_x,
            region_chunk_z,
        } = RegionAndOffset::from_chunk(chunk_x, chunk_z);

        let mut buf;
        let buf = if let Some(buf) = self.cache.get_mut(&(region_x, region_z)) {
            buf
        } else {
            let region_path = self.region_path(region_x, region_z);

            let mut region_file = match self.zip_archive.by_name(&region_path) {
                Ok(x) => x,
                Err(ZipError::FileNotFound) => {
                    return Err(ChunkLoadError::RegionNotFound { region_x, region_z })
                }
                Err(ZipError::Io(io_error)) => return Err(ChunkLoadError::ReadError { io_error }),
                Err(e) => panic!("Unhandled zip error: {}", e),
            };

            let uncompressed_size = region_file.size();
            buf = Vec::with_capacity(uncompressed_size as usize);
            region_file.read_to_end(&mut buf)?;

            // Insert into cache
            self.cache.insert((region_x, region_z), buf.clone());

            &mut buf
        };

        // Warning: the zip archive will not be updated with any writes!
        // Any writes made by AnvilRegion will only affect the in-memory cache
        // AnvilRegion needs Read+Seek+Write access to the reader
        // But ZipArchive only provides Read access to the compressed files
        // So we uncompress the file into memory, and pass the in-memory buffer
        // to AnvilRegion
        let mut region = AnvilRegion::new(Cursor::new(buf))?;

        region.read_chunk(region_chunk_x, region_chunk_z)
    }

    pub fn save_chunk(
        &mut self,
        _chunk_x: i32,
        _chunk_z: i32,
        _chunk_compound_tag: CompoundTag,
    ) -> Result<(), ChunkSaveError> {
        panic!("Writing to ZIP archives is not supported");
    }
}

impl ZipChunkProvider<File> {
    pub fn file<P: AsRef<Path>>(path: P) -> Result<Self, ZipProviderError> {
        let file = OpenOptions::new()
            .write(false)
            .read(true)
            .create(false)
            .open(path)?;

        Self::new(file)
    }
}

impl<R: Read + Seek> AnvilChunkProvider for ZipChunkProvider<R> {
    fn load_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Result<CompoundTag, ChunkLoadError> {
        self.load_chunk(chunk_x, chunk_z)
    }
    fn save_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_compound_tag: CompoundTag,
    ) -> Result<(), ChunkSaveError> {
        self.save_chunk(chunk_x, chunk_z, chunk_compound_tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_empty_buffer_as_zip() {
        // Try to read an empty buffer as a zip file
        let zip = b"";

        let z = ZipChunkProvider::new(Cursor::new(zip));

        match z.err().unwrap() {
            ZipProviderError::Zip(ZipError::InvalidArchive("Invalid zip header")) => {}
            e => panic!("Expected `Zip` but got `{:?}`", e),
        }
    }

    #[test]
    fn read_small_valid_zip() {
        // Smallest possible valid zip file:
        let zip = b"\x50\x4B\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

        // Reading works but since it has zero entries, the region/ folder
        // does not exist
        let z = ZipChunkProvider::new(Cursor::new(zip));

        match z {
            Err(ZipProviderError::RegionFolderNotFound) => {}
            e => panic!("Expected `RegionFolderNotFound` but got `{:?}`", e),
        }
    }

    #[test]
    fn read_zip_with_empty_region_folder() {
        let zip_file = Path::new("test/empty_region.zip");
        assert!(zip_file.exists());

        let mut z = ZipChunkProvider::file(zip_file).unwrap();
        let err = z.load_chunk(0, 0).unwrap_err();

        match err {
            ChunkLoadError::RegionNotFound {
                region_x: 0,
                region_z: 0,
            } => {}
            e => panic!("Expected `RegionNotFound` but got `{:?}`", e),
        }
    }

    #[test]
    fn read_zip_with_region_file() {
        let zip_file = Path::new("test/region.zip");
        assert!(zip_file.exists());

        let mut z = ZipChunkProvider::file(zip_file).unwrap();
        let compound_tag = z.load_chunk(15, 3).unwrap();
        let level_tag = compound_tag.get_compound_tag("Level").unwrap();

        assert_eq!(level_tag.get_i32("xPos").unwrap(), 15);
        assert_eq!(level_tag.get_i32("zPos").unwrap(), 3);
    }
}
