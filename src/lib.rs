use std::fs::{File, OpenOptions};
use std::io::Error;
use std::mem;
use std::path::Path;

/// Amount of chunks in region.
const REGION_CHUNKS: usize = 1024;
/// Region header length.
const REGION_CHUNKS_METADATA_LENGTH: u64 = (mem::size_of::<ChunkMetadata>() * REGION_CHUNKS) as u64;

/// Region represents a 32x32 group of chunks.
pub struct Region {
    /// File in which region are stored.
    file: File,
    /// Array of chunks metadata.
    chunks_metadata: [ChunkMetadata; REGION_CHUNKS],
}

/// Chunk metadata are stored in header.
#[derive(Copy, Clone, Default)]
pub struct ChunkMetadata {
    /// Position offset from file start at which starts chunk data.
    seek_offset: u32,
    /// Last time chunk was modified.
    last_modified_timestamp: u32,
}

/// Compression scheme used for chunk.
pub enum ChunkCompressionScheme {
    Gzip = 1,
    /// In practice, you will only ever encounter chunks compressed using zlib.
    Zlib = 2,
}

impl Region {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;

        let metadata = file.metadata()?;

        // If necessary, expand the file length to the length of the header.
        if REGION_CHUNKS_METADATA_LENGTH > metadata.len() {
            file.set_len(REGION_CHUNKS_METADATA_LENGTH)?;
        }

        let region = Region {
            file,
            chunks_metadata: [Default::default(); REGION_CHUNKS_SIZE],
        };

        Ok(region)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Region, REGION_CHUNKS_METADATA_LENGTH};
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_header_write() {
        let file = NamedTempFile::new().unwrap();
        let region = Region::new(file.path()).unwrap();

        assert_eq!(
            region.file.metadata().unwrap().len(),
            REGION_CHUNKS_METADATA_LENGTH
        )
    }
}
