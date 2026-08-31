use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PersistencyManager {
    path_to_db_root: PathBuf,
}

impl PersistencyManager {
    pub fn new(path_to_db_root: PathBuf) -> PersistencyManager {
        PersistencyManager { path_to_db_root }
    }

    pub fn read_file(&self, file_path: impl AsRef<Path>) -> io::Result<String> {
        let path = self.path_to_db_root.join(file_path.as_ref());
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn write_file(
        &self,
        file_path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> io::Result<()> {
        let path = self.path_to_db_root.join(file_path.as_ref());
        let mut file = File::create(path)?;
        file.write_all(contents.as_ref())?;
        Ok(())
    }
}