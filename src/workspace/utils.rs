use std::env;
use std::path::{Component, Path, PathBuf};

pub fn canonicalize<T: AsRef<Path>>(path: T) -> PathBuf {
    let mut buf = PathBuf::new();
    for v in path.as_ref().components() {
        match v {
            Component::RootDir => panic!("Does not supports root component in assets path."),
            Component::Prefix(_) => panic!("Does not supports prefix component in assets path."),
            Component::CurDir => continue,
            Component::ParentDir => assert!(buf.pop(), "Trying to access out of assets folder."),
            Component::Normal(v) => buf.push(v),
        }
    }
    buf
}

pub fn current_exe() -> PathBuf {
    let dir = env::current_exe().unwrap();
    dir.read_link().unwrap_or(dir)
}

pub fn current_exe_dir() -> PathBuf {
    let dir = env::current_exe().unwrap();
    dir.read_link().unwrap_or(dir).parent().unwrap().to_owned()
}
