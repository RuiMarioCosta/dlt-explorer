use std::path::PathBuf;

#[derive(Debug)]
pub struct Dlt<'a> {
    paths: Vec<PathBuf>,
    filter: Option<PathBuf>,

    data: Vec<u8>,
    patterns: Vec<&'a str>,
}

impl<'a> Dlt<'a> {
    pub fn new(paths: Vec<PathBuf>, filter: Option<PathBuf>) -> Self {
        let path = &paths[0];
        let data = std::fs::read(path).unwrap();

        let _a = &data[..4];

        Self {
            paths,
            filter,
            data,
            patterns: vec!["asdf"],
        }
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn filter(&self) -> &Option<PathBuf> {
        &self.filter
    }

    pub fn patterns(&self) -> &Vec<&'a str> {
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

        let _result = Dlt::new(paths, None);
    }
}
