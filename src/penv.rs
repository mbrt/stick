use std::path::PathBuf;

const PRJ_FILE: &'static str = ".stick.cfg";

pub fn root() -> Option<PathBuf> {
    let mut path = PathBuf::from(".");
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
    Some(path)
}

pub fn issues_dir() -> Option<PathBuf> {
    let mut root = match root() {
        Some(r) => r,
        None => {
            return None;
        }
    };
    root.push("issues");
    Some(root)
}

pub fn state_dir(name: &str) -> Option<PathBuf> {
    let mut root = match root() {
        Some(r) => r,
        None => {
            return None;
        }
    };
    root.push(name);
    Some(root)
}
