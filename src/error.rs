use crate::position::RegionChunkPosition;
use nbt::decode::TagDecodeError;
use std::{error::Error, fmt::Display, io};

/// Possible errors while loading the chunk.
#[derive(Debug)]
pub enum ChunkReadError {
    /// Chunk at specified coordinates inside region not found.
    ChunkNotFound { position: RegionChunkPosition },
    /// Chunk length overlaps declared maximum.
    ///
    /// This should not occur under normal conditions.
    ///
    /// Region file are corrupted.
    LengthExceedsMaximum {
        /// Chunk length.
        length: u32,
        /// Chunk maximum expected length.
        maximum_length: u32,
    },
    /// Currently are only 2 types of compression: Gzip and Zlib.
    ///
    /// This should not occur under normal conditions.
    ///
    /// Region file are corrupted or was introduced new compression type.
    UnsupportedCompressionScheme {
        /// Compression scheme type id.
        compression_scheme: u8,
    },
    /// I/O Error which happened while were reading chunk data from region file.
    IOError { io_error: io::Error },
    /// Error while decoding binary data to NBT tag.
    ///
    /// This should not occur under normal conditions.
    ///
    /// Region file are corrupted or a developer error in the NBT library.
    TagDecodeError { tag_decode_error: TagDecodeError },
}

impl From<io::Error> for ChunkReadError {
    fn from(io_error: io::Error) -> Self {
        ChunkReadError::IOError { io_error }
    }
}

impl From<TagDecodeError> for ChunkReadError {
    fn from(tag_decode_error: TagDecodeError) -> Self {
        ChunkReadError::TagDecodeError { tag_decode_error }
    }
}

impl Error for ChunkReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ChunkReadError::*;
        match self {
            IOError { io_error } => Some(io_error),
            TagDecodeError { tag_decode_error } => Some(tag_decode_error),
            _ => None,
        }
    }
}

impl Display for ChunkReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ChunkReadError::*;
        match self {
            ChunkNotFound { position } => {
                write!(f, "Chunk {}, {} not found", position.x, position.z)
            }
            LengthExceedsMaximum {
                length,
                maximum_length,
            } => write!(
                f,
                "Chunk lenght of {} exceeds maximum ({})",
                length, maximum_length
            ),
            UnsupportedCompressionScheme { compression_scheme } => {
                write!(f, "Unsupported compression scheme: {}", compression_scheme)
            }
            IOError { .. } => write!(f, "IO Error"),
            TagDecodeError { .. } => write!(f, "Failed to decode nbt"),
        }
    }
}

/// Possible errors while saving the chunk.
#[derive(Debug)]
pub enum ChunkWriteError {
    /// Chunk length exceeds 1 MB.
    ///
    /// This should not occur under normal conditions.
    LengthExceedsMaximum {
        /// Chunk length.
        length: u32,
    },
    /// I/O Error which happened while were writing chunk data to region.
    IOError { io_error: io::Error },
}

impl From<io::Error> for ChunkWriteError {
    fn from(io_error: io::Error) -> Self {
        ChunkWriteError::IOError { io_error }
    }
}

impl Error for ChunkWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChunkWriteError::IOError { io_error } => Some(io_error),
            _ => None,
        }
    }
}

impl Display for ChunkWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ChunkWriteError::*;
        match self {
            LengthExceedsMaximum { length } => {
                write!(f, "Chunk lenght of {} exceeds maximum (1mb)", length)
            }
            IOError { .. } => write!(f, "IO Error"),
        }
    }
}
