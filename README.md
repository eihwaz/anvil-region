anvil-region
============
[![crates.io](https://img.shields.io/crates/v/anvil-region.svg)](https://crates.io/crates/anvil-region)
[![Build Status](https://travis-ci.com/eihwaz/anvil-region.svg?branch=master)](https://travis-ci.com/eihwaz/anvil-region)
[![codecov](https://codecov.io/gh/eihwaz/anvil-region/branch/master/graph/badge.svg)](https://codecov.io/gh/eihwaz/anvil-region)

Region file format storage for chunks

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
anvil-region = "0.8"
```

## Example

#### Read

```rust
use anvil_region::provider::{FolderRegionProvider, RegionProvider};
use anvil_region::position::{RegionPosition, RegionChunkPosition};

let provider = FolderRegionProvider::new("test/region");

let region_position = RegionPosition::from_chunk_position(4, 2);
let region_chunk_position = RegionChunkPosition::from_chunk_position(4, 2);

let mut region = provider.get_region(region_position).unwrap();

let chunk_compound_tag = region.read_chunk(region_chunk_position).unwrap();
let level_compound_tag = chunk_compound_tag.get_compound_tag("Level").unwrap();

assert_eq!(level_compound_tag.get_i32("xPos").unwrap(), 4);
assert_eq!(level_compound_tag.get_i32("zPos").unwrap(), 2);
```

#### Write

```rust
use anvil_region::provider::{FolderRegionProvider, RegionProvider};
use nbt::CompoundTag;
use anvil_region::position::{RegionPosition, RegionChunkPosition};

let provider = FolderRegionProvider::new("test/region");

let region_position = RegionPosition::from_chunk_position(31, 16);
let region_chunk_position = RegionChunkPosition::from_chunk_position(31, 16);

let mut region = provider.get_region(region_position).unwrap();

let mut chunk_compound_tag = CompoundTag::new();
let mut level_compound_tag = CompoundTag::new();

// To simplify example we add only coordinates.
// Full list of required tags https://minecraft.gamepedia.com/Chunk_format.
level_compound_tag.insert_i32("xPos", 31);
level_compound_tag.insert_i32("zPos", 16);

chunk_compound_tag.insert_compound_tag("Level", level_compound_tag);

region.write_chunk(region_chunk_position, chunk_compound_tag);
```
