use crate::models::LiteNode;
use rkyv::{
    rancor::{Error, Source},
    Archive, Deserialize, Serialize,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tempfile::NamedTempFile;

pub struct FileCache {
    base_path: PathBuf,
}

impl FileCache {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let path = app_handle.path().app_data_dir().unwrap().join("cache");
        fs::create_dir_all(&path).unwrap();
        Self { base_path: path }
    }

    pub fn save(&self, filename: &str, nodes: &Vec<LiteNode>) -> Result<(), Error> {
        let target_path = self.base_path.join(filename);

        let mut temp_file = NamedTempFile::new_in(&self.base_path).map_err(Error::new)?;
        let bytes = rkyv::to_bytes::<Error>(nodes)?;
        temp_file.write_all(&bytes).map_err(Error::new)?;
        temp_file
            .persist(target_path)
            .map_err(|e| Error::new(e.error))?;

        Ok(())
    }

    pub fn load(&self, filename: &str) -> Result<Vec<LiteNode>, Error> {
        let target_path = self.base_path.join(filename);
        let bytes = fs::read(&target_path).map_err(Error::new)?;
        rkyv::from_bytes::<Vec<LiteNode>, Error>(&bytes)
    }
}
