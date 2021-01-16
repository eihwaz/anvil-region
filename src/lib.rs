//! Region file format storage for chunks.
//!
//! More information about format can be found https://wiki.vg/Region_Files.
//!
//! # Example
//!
//! ## Read
//!
//! ```
//! use anvil_region::provider::{FolderRegionProvider, RegionProvider};
//!
//! let provider = FolderRegionProvider::new("test/region");
//! let mut region = provider.get_region(0, 0).unwrap();
//!
//! let chunk_compound_tag = region.read_chunk(4, 2).unwrap();
//! let level_compound_tag = chunk_compound_tag.get_compound_tag("Level").unwrap();
//!
//! assert_eq!(level_compound_tag.get_i32("xPos").unwrap(), 4);
//! assert_eq!(level_compound_tag.get_i32("zPos").unwrap(), 2);
//! ```
//!
//! ## Write
//!
//! ```
//! use anvil_region::provider::{FolderRegionProvider, RegionProvider};
//! use nbt::CompoundTag;
//!
//! let provider = FolderRegionProvider::new("test/region");
//! let mut region = provider.get_region(0, 0).unwrap();
//!
//! let mut chunk_compound_tag = CompoundTag::new();
//! let mut level_compound_tag = CompoundTag::new();
//!
//! // To simplify example we add only coordinates.
//! // Full list of required tags https://minecraft.gamepedia.com/Chunk_format.
//! level_compound_tag.insert_i32("xPos", 31);
//! level_compound_tag.insert_i32("zPos", 16);
//!
//! chunk_compound_tag.insert_compound_tag("Level", level_compound_tag);
//!
//! region.write_chunk(31, 16, chunk_compound_tag);
//! ```
pub mod error;
pub mod provider;
pub mod region;
