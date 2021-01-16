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
//!
//! let chunk_x = 4;
//! let chunk_z = 2;
//!
//! let region_x = chunk_x >> 31;
//! let region_z = chunk_z >> 31;
//!
//! let region_chunk_x = (chunk_x & 31) as u8;
//! let region_chunk_z = (chunk_z & 31) as u8;
//!
//! let mut region = provider.get_region(region_x, region_z).unwrap();
//!
//! let chunk_compound_tag = region.read_chunk(region_chunk_x, region_chunk_z).unwrap();
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
//!
//! let chunk_x = 31;
//! let chunk_z = 16;
//!
//! let region_x = chunk_x >> 31;
//! let region_z = chunk_z >> 31;
//!
//! let region_chunk_x = (chunk_x & 31) as u8;
//! let region_chunk_z = (chunk_z & 31) as u8;
//!
//! let mut region = provider.get_region(region_x, region_z).unwrap();
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
//! region.write_chunk(region_chunk_x, region_chunk_z, chunk_compound_tag);
//! ```
pub mod error;
pub mod provider;
pub mod region;
