use byteorder::{BigEndian, ReadBytesExt};
use flate2::read;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Error, Read, Seek, SeekFrom};
use std::path::Path;

/// Amount of chunks in region.
const REGION_CHUNKS: usize = 1024;
/// Length of chunks metadata in region.
const REGION_CHUNKS_METADATA_LENGTH: usize = 2 * REGION_CHUNKS;
/// Region header length in bytes.
const REGION_HEADER_BYTES_LENGTH: u64 = 8 * REGION_CHUNKS as u64;
/// Region sector length in bytes.
const REGION_SECTOR_BYTES_LENGTH: u16 = 4096;

/// Region represents a 32x32 group of chunks.
pub struct Region {
    /// File in which region are stored.
    file: File,
    /// Array of chunks metadata.
    chunks_metadata: [ChunkMetadata; REGION_CHUNKS],
}

/// Chunk metadata are stored in header.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct ChunkMetadata {
    /// Sector index from which starts chunk data.
    sector_index: u32,
    /// Amount of sectors used to store chunk.
    sectors: u8,
    /// Last time chunk was modified.
    last_modified_timestamp: u32,
}

impl ChunkMetadata {
    pub fn new(sector_index: u32, sectors: u8, last_modified_timestamp: u32) -> Self {
        ChunkMetadata {
            sector_index,
            sectors,
            last_modified_timestamp,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sectors == 0
    }
}

impl Region {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;

        // If necessary, expand the file length to the length of the header.
        if REGION_HEADER_BYTES_LENGTH > file.metadata()?.len() {
            file.set_len(REGION_HEADER_BYTES_LENGTH)?;
        }

        let chunks_metadata = Self::read_header(&mut file)?;

        let region = Region {
            file,
            chunks_metadata,
        };

        Ok(region)
    }

    fn read_header(file: &mut File) -> Result<[ChunkMetadata; REGION_CHUNKS], Error> {
        let mut chunks_metadata = [Default::default(); REGION_CHUNKS];
        let mut values = [0u32; REGION_CHUNKS_METADATA_LENGTH];

        for index in 0..REGION_CHUNKS_METADATA_LENGTH {
            values[index] = file.read_u32::<BigEndian>()?;
        }

        for index in 0..REGION_CHUNKS {
            let last_modified_timestamp = values[REGION_CHUNKS + index];
            let offset = values[index];

            let sector_index = offset >> 8;
            let sectors = (offset & 0xFF) as u8;

            let metadata = ChunkMetadata {
                sector_index,
                sectors,
                last_modified_timestamp,
            };

            chunks_metadata[index] = metadata;
        }

        return Ok(chunks_metadata);
    }

    pub fn read_chunk_data(&mut self, x: u8, z: u8) -> Result<Vec<u8>, Error> {
        assert!(32 > x, "Region chunk x coordinate out of bounds");
        assert!(32 > z, "Region chunk y coordinate out of bounds");

        let metadata = self.get_metadata(x, z);

        if metadata.is_empty() {
            return Ok(Vec::new());
        }

        let seek_offset = metadata.sector_index as u64 * REGION_SECTOR_BYTES_LENGTH as u64;
        self.file.seek(SeekFrom::Start(seek_offset))?;

        let maximum_length = metadata.sectors as u32 * REGION_SECTOR_BYTES_LENGTH as u32;
        let length = self.file.read_u32::<BigEndian>()?;

        assert!(
            maximum_length >= length,
            "Chunk of length {} exceeds maximum {}",
            length,
            maximum_length
        );

        let compression_scheme = self.file.read_u8()?;
        let mut compressed_buffer = vec![0u8; (length - 1) as usize];
        self.file.read_exact(&mut compressed_buffer)?;

        let cursor = Cursor::new(&compressed_buffer);
        let mut buffer = Vec::new();

        match compression_scheme {
            1 => {
                read::GzDecoder::new(cursor).read_to_end(&mut buffer)?;
            }
            2 => {
                read::ZlibDecoder::new(cursor).read_to_end(&mut buffer)?;
            }
            _ => panic!(
                "Unsupported compression scheme of type {}",
                compression_scheme
            ),
        }

        Ok(buffer)
    }

    fn get_metadata(&self, x: u8, z: u8) -> ChunkMetadata {
        self.chunks_metadata[(x + z) as usize * 32]
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChunkMetadata, Region, REGION_HEADER_BYTES_LENGTH};
    use std::io::Read;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_header_write() {
        let mut file = NamedTempFile::new().unwrap();
        let region = Region::new(file.path()).unwrap();
        let file_length = region.file.metadata().unwrap().len();

        assert_eq!(file_length, REGION_HEADER_BYTES_LENGTH);
    }

    #[test]
    fn test_empty_region_init() {
        let mut file = NamedTempFile::new().unwrap();
        let region = Region::new(file.path()).unwrap();

        let mut vec = Vec::new();
        file.read_to_end(&mut vec).unwrap();

        assert_eq!(vec, include_bytes!("../test/empty_region.mca").to_vec());
    }

    #[test]
    fn test_header_read() {
        let expected_data = vec![
            ChunkMetadata::new(692, 1, 1570215596),
            ChunkMetadata::new(772, 1, 1570215597),
            ChunkMetadata::new(875, 1, 1570215597),
            ChunkMetadata::new(991, 1, 1570215597),
            ChunkMetadata::new(696, 1, 1570215596),
            ChunkMetadata::new(795, 1, 1570215597),
            ChunkMetadata::new(281, 1, 1570215597),
            ChunkMetadata::new(1018, 1, 1570215597),
            ChunkMetadata::new(735, 1, 1570215596),
            ChunkMetadata::new(812, 1, 1570215597),
        ];

        let path = Path::new("./test/region.mca");
        assert!(path.exists());

        let region = Region::new(path).unwrap();

        for (index, expected_chunk_metadata) in expected_data.iter().enumerate() {
            let chunk_metadata = region.chunks_metadata[256 + index];

            assert_eq!(&chunk_metadata, expected_chunk_metadata);
        }
    }

    #[test]
    fn test_read_chunk_data_existing() {
        let path = Path::new("./test/region.mca");
        assert!(path.exists());

        let mut region = Region::new(path).unwrap();
        let vec = region.read_chunk_data(4, 4).unwrap();

        assert_eq!(vec.len(), 28061);
    }

    #[test]
    fn test_read_chunk_data_empty() {
        let path = Path::new("./test/empty_region.mca");
        assert!(path.exists());

        let mut region = Region::new(path).unwrap();
        let vec = region.read_chunk_data(0, 0).unwrap();

        assert_eq!(vec.len(), 0);
    }

}
