use std::path::{Path, PathBuf};

const PRJ_FILE: &'static str = ".stick.cfg";
const ISSUES_DIR: &'static str = "issues";
const STATES_DIR: &'static str = "state";

pub struct Environment {
    root: PathBuf,
}


impl Environment {
    pub fn new() -> Option<Self> {
        Self::from_path(".")
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let mut path = PathBuf::from(path.as_ref());
        loop {
            // check if directory
            if !path.is_dir() {
                return None;
            }
            let mut prjfile = PathBuf::new();
            prjfile.push(PRJ_FILE);
            if prjfile.is_file() {
                break;
            }
            path.push("..");
        }
        Some(Environment { root: path })
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn issues_dir(&self) -> PathBuf {
        let mut res = self.root.clone();
        res.push(ISSUES_DIR);
        res
    }

    pub fn state_dir<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        let mut res = self.root.clone();
        res.push(STATES_DIR);
        res.push(name.as_ref());
        res
    }
}
