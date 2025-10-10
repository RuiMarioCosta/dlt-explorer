use std::path::PathBuf;

#[derive(Debug)]
pub struct Dlt {
    paths: Vec<PathBuf>,
    filter: Option<PathBuf>,
}

impl Dlt {
    pub fn new(paths: Vec<PathBuf>, filter: Option<PathBuf>) -> Self {
        Self { paths, filter }
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn filter(&self) -> &Option<PathBuf> {
        &self.filter
    }
}
