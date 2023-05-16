use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[cfg(unix)]
mod localization {
    pub const ROOT: &str = "/";
    pub const SLASH: char = '/';
    pub const SLASH_STR: &str = ROOT;
}

#[cfg(windows)]
mod localization {
    pub const ROOT: &str = "C:\\";
    pub const SLASH: char = '\\';
    pub const SLASH_STR: &str = "\\";
}

use localization::{ROOT, SLASH, SLASH_STR};

const RC: char = char::REPLACEMENT_CHARACTER; // '�'
const BS: char = '\\';
const FS: char = '/';
const UP: &str = "..";

#[derive(Clone, PartialEq, Debug, Default)]
pub struct OsPath {
    components: Vec<String>,
    absolute: bool,
    directory: bool,
    path: PathBuf,
}

impl OsPath {
    pub fn new() -> Self {
        Self::default()
    }

    /// Traverses the components of the path and and resolves any `..` components.
    /// This cannot be done automatically because ".." may be desireable in some cases.
    pub fn resolve(&mut self) {
        let mut new_vec: Vec<String> = Vec::new();
        for c in &self.components {
            if c != UP {
                new_vec.push(c.clone());
            } else {
                new_vec.pop();
            }
        }
        self.components = new_vec;
        self.path = Self::build_pathbuf(&self.components, self.absolute);
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut new_self = self.clone();
        let path = Self::build_self(path);
        Self::merge_paths(&mut new_self, path);
        new_self.path = Self::build_pathbuf(&new_self.components, new_self.absolute);
        new_self
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        let path = Self::build_self(path);
        Self::merge_paths(self, path);
        self.path = Self::build_pathbuf(&self.components, self.absolute);
    }
}

impl OsPath {
    pub fn absolute(&self) -> bool {
        self.absolute
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn is_file(&self) -> bool {
        !self.directory
    }

    pub fn is_dir(&self) -> bool {
        self.directory
    }

    pub fn name(&self) -> Option<&String> {
        if self.components.len() > 0 {
            return self.components.last();
        }
        None
    }

    pub fn parent(&self) -> Option<Self> {
        if self.components.len() < 2 {
            return None;
        }
        let i = self.components.len() - 1;
        let mut new_self = self.clone();
        new_self.components.truncate(i);
        new_self.path = Self::build_pathbuf(&new_self.components, new_self.absolute);
        Some(new_self)
    }
}

impl OsPath {
    pub fn to_string(&self) -> String {
        match self.absolute {
            true => ROOT.to_string() + &self.components.join(SLASH_STR),
            false => self.components.join(SLASH_STR),
        }
    }

    pub fn to_pathbuf(&self) -> PathBuf {
        let path = self.to_path();
        path.to_owned()
    }

    pub fn to_path(&self) -> &Path {
        self.path.as_path()
    }
}

impl OsPath {
    fn build_self<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_string_lossy().to_string();
        let absolute = path.starts_with(ROOT);
        let directory = if path.ends_with(SLASH) || path.ends_with(UP) {
            true
        } else {
            false
        };
        let clean: String = path
            .chars()
            .map(|c| if c == BS || c == FS { RC } else { c })
            .collect();
        let components: Vec<String> = clean
            .split(RC)
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .collect();
        let path = Self::build_pathbuf(&components, absolute);
        Self {
            components,
            absolute,
            directory,
            path,
        }
    }

    fn build_pathbuf(components: &Vec<String>, absolute: bool) -> PathBuf {
        let mut path = PathBuf::new();
        if absolute {
            path.push(ROOT);
        }
        for c in components {
            path.push(c);
        }
        path
    }

    fn merge_paths(first: &mut Self, mut second: Self) {
        if second.components.len() == 0 {
            return;
        }
        if !first.directory && second.components.first().unwrap() == UP {
            first.components.pop();
            first.components.pop();
            second.components.remove(0);
        }
        for c in second.components {
            if c == UP {
                first.components.pop();
                continue;
            }
            first.components.push(c);
        }
        first.directory = second.directory;
    }
}

impl From<&OsPath> for OsPath {
    fn from(p: &OsPath) -> Self {
        p.clone()
    }
}

impl From<&str> for OsPath {
    fn from(s: &str) -> Self {
        Self::build_self(s)
    }
}

impl From<String> for OsPath {
    fn from(s: String) -> Self {
        Self::build_self(&s)
    }
}

impl From<PathBuf> for OsPath {
    fn from(p: PathBuf) -> Self {
        Self::build_self(&p)
    }
}

impl AsRef<Path> for OsPath {
    fn as_ref(&self) -> &Path {
        self.to_path()
    }
}

impl AsRef<OsStr> for OsPath {
    fn as_ref(&self) -> &OsStr {
        self.to_path().as_os_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let path = OsPath::new();
        assert_eq!(path.components.len(), 0);
        assert_eq!(path.absolute, false);
        assert_eq!(path.directory, false);
        assert_eq!(path.path, PathBuf::new());
    }

    #[test]
    fn test_build_self() {
        #[cfg(unix)]
        {
            let path = OsPath::build_self("/");
            assert_eq!(path.components.len(), 0);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, true);
            assert_eq!(path.path, PathBuf::from("/"));

            let path = OsPath::build_self("/a/b/c");
            assert_eq!(path.components.len(), 3);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, false);
            assert_eq!(path.path, PathBuf::from("/a/b/c"));

            let path = OsPath::build_self("/a/b/c/");
            assert_eq!(path.components.len(), 3);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, true);
            assert_eq!(path.path, PathBuf::from("/a/b/c/"));

            let path = OsPath::build_self("a/b/c");
            assert_eq!(path.components.len(), 3);
            assert_eq!(path.absolute, false);
            assert_eq!(path.directory, false);
            assert_eq!(path.path, PathBuf::from("a/b/c"));

            let path = OsPath::build_self("a/b/c/../../../d");
            println!("{:?}", path);
            assert_eq!(path.components.len(), 7);
            assert_eq!(path.absolute, false);
            assert_eq!(path.directory, false);
            assert_eq!(path.path, PathBuf::from("a/b/c/../../../d"));
        }

        #[cfg(windows)]
        {
            let path = OsPath::build_self("C:\\");
            assert_eq!(path.components.len(), 0);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, true);
            assert_eq!(path.path, PathBuf::from("C:\\"));

            let path = OsPath::build_self("C:\\a\\b\\c");
            assert_eq!(path.components.len(), 3);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, false);
            assert_eq!(path.path, PathBuf::from("C:\\a\\b\\c"));

            let path = OsPath::build_self("C:\\a\\b\\c\\");
            assert_eq!(path.components.len(), 3);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, true);
            assert_eq!(path.path, PathBuf::from("C:\\a\\b\\c\\"));

            let path = OsPath::build_self("C:\\a\\b\\c\\..\\..\\..\\d");
            assert_eq!(path.components.len(), 1);
            assert_eq!(path.absolute, true);
            assert_eq!(path.directory, false);
            assert_eq!(path.path, PathBuf::from("C:\\a\\b\\c\\..\\..\\..\\d"));
        }
    }
}
