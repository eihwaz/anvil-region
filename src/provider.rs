use crate::region::Region;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{fs, io};

pub trait RegionProvider<S> {
    fn get_region(&self, region_x: i32, region_y: i32) -> Result<Region<S>, io::Error>;
}

pub struct FolderRegionProvider<'a> {
    /// Folder where region files located.
    folder_path: &'a Path,
}

impl<'a> FolderRegionProvider<'a> {
    pub fn new(folder: &'a str) -> FolderRegionProvider<'a> {
        let folder_path = Path::new(folder);

        FolderRegionProvider { folder_path }
    }
}

impl<'a> RegionProvider<File> for FolderRegionProvider<'a> {
    fn get_region(&self, region_x: i32, region_z: i32) -> Result<Region<File>, io::Error> {
        if !self.folder_path.exists() {
            fs::create_dir(self.folder_path)?;
        }

        let region_name = format!("r.{}.{}.mca", region_x, region_z);
        let region_path = self.folder_path.join(region_name);

        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(region_path)?;

        Region::load(region_x, region_z, file)
    }
}
