use byteorder::{BigEndian, ReadBytesExt};
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::mem;
use std::path::Path;

/// Amount of chunks in region.
const REGION_CHUNKS: usize = 1024;
/// Length of chunks metadata in region.
const REGION_CHUNKS_METADATA_LENGTH: usize = 2 * REGION_CHUNKS;
/// Region header length in bytes.
const REGION_HEADER_BYTES_LENGTH: u64 = (mem::size_of::<ChunkMetadata>() * REGION_CHUNKS) as u64;

/// Region represents a 32x32 group of chunks.
pub struct Region {
    /// File in which region are stored.
    file: File,
    /// Array of chunks metadata.
    chunks_metadata: [ChunkMetadata; REGION_CHUNKS],
}

/// Chunk metadata are stored in header.
#[derive(Copy, Clone, Default, Debug)]
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
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;

        // If necessary, expand the file length to the length of the header.
        if REGION_HEADER_BYTES_LENGTH > file.metadata()?.len() {
            file.set_len(REGION_HEADER_BYTES_LENGTH)?
        }

        let chunks_metadata = Self::read_chunks_metadata(&mut file)?;

        let region = Region {
            file,
            chunks_metadata,
        };

        Ok(region)
    }

    fn read_chunks_metadata(file: &mut File) -> Result<[ChunkMetadata; REGION_CHUNKS], Error> {
        let mut chunks_metadata = [Default::default(); REGION_CHUNKS];
        let mut values = [0u32; REGION_CHUNKS_METADATA_LENGTH];

        for index in 0..REGION_CHUNKS_METADATA_LENGTH {
            values[index] = file.read_u32::<BigEndian>()?;
        }

        for index in 0..REGION_CHUNKS {
            let seek_offset = values[index];
            let last_modified_timestamp = values[REGION_CHUNKS + index];

            let metadata = ChunkMetadata {
                seek_offset,
                last_modified_timestamp,
            };

            chunks_metadata[index] = metadata;
        }

        return Ok(chunks_metadata);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Region, REGION_HEADER_BYTES_LENGTH};
    use std::fs::File;
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
    fn test_chunk_metadata_read() {
        let expected_data = vec![
            (177153u32, 1570215596u32),
            (197633, 1570215597),
            (224001, 1570215597),
            (253697, 1570215597),
            (178177, 1570215596),
            (203521, 1570215597),
            (71937, 1570215597),
            (260609, 1570215597),
            (188161, 1570215596),
            (207873, 1570215597),
        ];

        let path = Path::new("./test/region.mca");
        assert!(path.exists());

        let region = Region::new(path).unwrap();

        for (index, (seek_offset, last_modified_timestamp)) in expected_data.iter().enumerate() {
            let metadata = region.chunks_metadata[256 + index];

            assert_eq!(metadata.seek_offset, *seek_offset);
            assert_eq!(metadata.last_modified_timestamp, *last_modified_timestamp);
        }
    }

}
