use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::Result;
use memchr::memmem::Finder;


const DLT_DELIMITER: &[u8] = b"DLT";


#[derive(Debug)]
pub struct Dlt<'a> {
    paths: Vec<PathBuf>,
    filter: Option<PathBuf>,

    // data: Vec<u8>,
    patterns: Vec<&'a str>,
}

impl<'a> Dlt<'a> {
    pub fn from_files(paths: Vec<PathBuf>, filter: Option<PathBuf>) -> Result<Self> {
        let path = &paths[0];
        // let data = std::fs::read(path).unwrap();
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let finder = Finder::new(DLT_DELIMITER);
        let mut buf = Vec::new();
        let len = reader.read(&mut buf);

        // let _a = &data[..100];

        Ok(Self {
            paths,
            filter,
            // data,
            patterns: vec!["asdf"],
        })
    }

    pub fn _paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn _filter(&self) -> &Option<PathBuf> {
        &self.filter
    }

    pub fn _patterns(&self) -> &Vec<&'a str> {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn open_dlt_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(PathBuf::from("src/dlt/tests/testfile_control_messages.dlt"));
        let paths = vec![path];

        let result = Dlt::from_files(paths, None);

        assert!(result.is_ok());
    }
}
