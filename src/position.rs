use std::path::PathBuf;
use std::io;
use std::str::FromStr;
use std::num::ParseIntError;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct RegionPosition {
    pub x: i32,
    pub z: i32,
}

impl RegionPosition {
    pub fn new(x: i32, z: i32) -> RegionPosition {
        RegionPosition { x, z }
    }

    pub fn from_chunk_position(chunk_x: i32, chunk_z: i32) -> RegionPosition {
        let x = chunk_x >> 5;
        let z = chunk_z >> 5;

        RegionPosition::new(x, z)
    }

    pub fn from_filename(path: &PathBuf) -> Result<RegionPosition, io::Error> {
        // we can use lossy because of the bound check later
        let filename = path.file_name().unwrap_or_default().to_string_lossy();

        let parts: Vec<_> = filename.split('.').collect();

        let (x, z) = parse_coords(parts).map_err(|_| io::ErrorKind::InvalidInput)?;

        Ok(RegionPosition::new(x, z))
    }

    pub fn filename(self) -> String {
        format!("r.{}.{}.mca", self.x, self.z)
    }
}

fn parse_coords(parts: Vec<&str>) -> Result<(i32, i32), ParseIntError> {
    let correct_format =
        parts.len() != 4 ||
            parts[0] != "r" ||
            parts[3] != "mca";

    if correct_format {
        // to throw the error (cant instantiate from outside)
        i32::from_str("")?;
    }

    Ok((i32::from_str(parts[1])?,
        i32::from_str(parts[2])?))
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct RegionChunkPosition {
    pub x: u8,
    pub z: u8,
}

impl RegionChunkPosition {
    pub fn new(x: u8, z: u8) -> RegionChunkPosition {
        debug_assert!(32 > x, "Region chunk x coordinate out of bounds");
        debug_assert!(32 > z, "Region chunk z coordinate out of bounds");

        RegionChunkPosition { x, z }
    }

    pub fn from_chunk_position(chunk_x: i32, chunk_z: i32) -> RegionChunkPosition {
        let x = (chunk_x & 31) as u8;
        let z = (chunk_z & 31) as u8;

        RegionChunkPosition::new(x, z)
    }

    pub(crate) fn metadata_index(&self) -> usize {
        self.x as usize + self.z as usize * 32
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::position::RegionPosition;

    #[test]
    fn test_position_parse() {
        let mut path = PathBuf::new();
        path.set_file_name("r.0.0.mca");

        let pos = RegionPosition::from_filename(&path).unwrap();
        assert_eq!(RegionPosition{ x: 0, z: 0}, pos)
    }

    #[test]
    #[should_panic]
    fn test_position_parse_invalid_format() {
        let mut path = PathBuf::new();
        path.set_file_name("this is not a valid region.filename");

        RegionPosition::from_filename(&path).unwrap();
    }
}