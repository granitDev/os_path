//! Cross-platform intelligent path creation, resolution, and manipulation.
//!
//! # Path Normalization
//!
//! Path slashes are normalized to your platform's native path format at creation and modification. This resolves
//! PathBuf's issue of returning to you the exact string you passed to it, even if it's incorrect for the current
//! platform.
//! ```rust
//! // Standard Library
//! use std::path::PathBuf;
//!
//! let mut path_buf = PathBuf::from("\\foo\\bar\\baz.txt");
//! assert_eq!(path_buf.to_string_lossy(),"\\foo\\bar\\baz.txt");
//!
//! // OsPath
//! use os_path::OsPath;
//!
//! let mut os_path = OsPath::from("\\foo\\bar\\baz.txt");
//! assert_eq!(os_path.to_string(),"/foo/bar/baz.txt");
//! ```
//!
//!
//! # False Root Handling
//!
//!
//! ```rust
//! // Standard Library
//! use std::path::PathBuf;
//!
//! let mut path_buf = PathBuf::new();
//! path_buf.push("/foo/bar");
//! path_buf.push("/baz/pow.txt");
//! assert_eq!(path_buf.to_string_lossy(),"/baz/pow.txt");
//!
//! // OsPath
//! use os_path::OsPath;
//!
//! let mut os_path = OsPath::new();
//! os_path.push("/foo/bar");
//! os_path.push("/baz.txt");
//! assert_eq!(os_path.to_string(),"/foo/bar/baz.txt");
//! ```
//!
//! False root errors occur when you you attempt to join paths with leading slashes. In the above example we have
//! `/foo/bar` and we push() /baz.txt to it. With the standard libraries Path and PathBuf, you'll end up with `/baz.txt`
//! as your path. This is very counter intuitive, and requires extra code be written to strip the leading slash in order
//! to prevent this.
//!
//! Instead, OsPath will do what you expect, and return /foo/bar/baz.txt.
//!
//! And OsPath does this while still assuming at the start that both paths were absolute. If you queried either path
//! beforehand, they would both return true for `is_absolute()`. However, when you joined the two paths, OsPath correctly
//! assumes the second path is relative to the first, and joins them correctly.
//!
//! > Note that this is not a problem on Windows, as attempting to join any path starting with `C:\` is nonsensical,
//! > while joinging a path prefixed with `/` or `\\` is not.
//!
//! //! # Path Resolution and Traversal
//!
//! If you `join()` or `push()` a path that starts with `..`, OsPath will traverse the path, and build the correct path.
//!
//! ```rust
//! // Standard Library
//! use std::path::PathBuf;
//!
//! let mut path_buf = PathBuf::new();
//! path_buf.push("/foo/bar");
//! path_buf.push("../baz.txt");
//! assert_eq!(path_buf.to_string_lossy(),"/foo/bar/../baz.txt");
//!
//! // OsPath
//! use os_path::OsPath;
//!
//! let mut os_path = OsPath::new();
//! os_path.push("/foo/bar/");
//! os_path.push("../baz.txt");
//! assert_eq!(os_path.to_string(),"/foo/baz.txt");
//! ```
//!
//! //! OsPath can handle multiple `..` in a row, and will traverse the path correctly.
//!
//! ```rust
//! use os_path::OsPath;
//!
//! let mut os_path = OsPath::new();
//! os_path.push("/foo/bar/baz/");
//! os_path.push("../../pow.txt");
//! assert_eq!(os_path.to_string(),"/foo/pow.txt");
//! ```
//!
//! And, if your path ends in a file, and you `join()` or `push()` a path that starts with `..`, OsPath will traverse the
//! path, and build the correct path, skipping over the file.
//!
//! ```rust
//! use os_path::OsPath;
//!
//! let mut os_path = OsPath::new();
//! os_path.push("/foo/bar/baz.txt");
//! os_path.push("../pow.txt");
//! assert_eq!(os_path.to_string(),"/foo/pow.txt");
//! ```
//!
//! # File And Directory Handling
//!
//! If the path ends in a `/` or `\\` OsPath assumes this is a directory, otherwise it's a file.
//!

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::OsStr;
use std::fmt;
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

/// An intelligent path type that can be used in place of `std::path::PathBuf`.
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

    /// Creates a new OsPath from the existing one, and joins the path to it.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("/foo/bar/");
    /// let new_os_path = os_path.join("/baz.txt");
    /// assert_eq!(new_os_path.to_string(),"/foo/bar/baz.txt");
    /// ```
    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut new_self = self.clone();
        let path = Self::build_self(path);
        Self::merge_paths(&mut new_self, path);
        new_self.path = Self::build_pathbuf(&new_self.components, new_self.absolute);
        new_self
    }

    /// Mutates self by appending the supplied path to it.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let mut os_path = OsPath::from("/foo/bar/");
    /// os_path.push("/baz.txt");
    /// assert_eq!(os_path.to_string(),"/foo/bar/baz.txt");
    /// ```
    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        let path = Self::build_self(path);
        Self::merge_paths(self, path);
        self.path = Self::build_pathbuf(&self.components, self.absolute);
    }

    /// Traverses the components of the path and and resolves any `..` components.
    /// This cannot be done automatically because ".." may be desireable in some cases.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let mut os_path = OsPath::from("/foo/bar/baz/../pow.txt");
    /// assert_eq!(os_path.to_string(),"/foo/bar/baz/../pow.txt");
    ///
    /// os_path.resolve();
    /// assert_eq!(os_path.to_string(),"/foo/bar/pow.txt");
    /// ```
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
}

impl OsPath {
    /// Returns true if the path is absolute.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("/absolute/path/");
    /// assert!(os_path.is_absolute());
    ///
    /// let os_path = OsPath::from("not/absolute/path/");
    /// assert!(!os_path.is_absolute());
    /// ```
    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    /// Returns true if the path exists.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("src/lib.rs");
    /// assert!(os_path.exists());
    /// ```
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Returns true if the last item is a file.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("src/lib.rs");
    /// assert!(os_path.is_file());
    /// ```
    pub fn is_file(&self) -> bool {
        !self.directory
    }

    /// Returns true if the last item is a directory.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("src/");
    /// assert!(os_path.is_dir());
    /// ```
    pub fn is_dir(&self) -> bool {
        self.directory
    }

    /// Returns the last item as a String.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("src/lib.rs");
    /// assert_eq!(os_path.name().unwrap().to_string(), "lib.rs");
    /// ```
    pub fn name(&self) -> Option<&String> {
        if !self.components.is_empty() {
            return self.components.last();
        }
        None
    }

    /// Returns the extension of the file if it has one.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("src/lib.rs");
    /// assert_eq!(os_path.name().unwrap().to_string(), "lib.rs");
    /// ```
    pub fn extension(&self) -> Option<String> {
        if self.is_file() {
            return Some(self.name()?.split('.').last()?.to_string());
        }
        None
    }

    /// Returns the extension of the file if it has one.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("foo/bar/baz/pow.txt");
    /// assert_eq!(os_path.parent().unwrap().to_string(), "foo/bar/baz/");
    /// ```
    pub fn parent(&self) -> Option<Self> {
        if self.components.len() < 2 && !self.absolute {
            return None;
        }
        let i = self.components.len() - 1;
        let mut new_self = self.clone();
        new_self.components.truncate(i);
        new_self.path = Self::build_pathbuf(&new_self.components, new_self.absolute);
        new_self.directory = true;
        Some(new_self)
    }
}

impl OsPath {
    fn build_string(&self) -> String {
        match (self.absolute, self.directory) {
            (true, true) => ROOT.to_string() + &self.components.join(SLASH_STR) + SLASH_STR,
            (true, false) => ROOT.to_string() + &self.components.join(SLASH_STR),
            (false, false) => self.components.join(SLASH_STR),
            (false, true) => self.components.join(SLASH_STR) + SLASH_STR,
        }
    }

    /// Returns the path as a PathBuf.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("/foo/bar/baz.txt");
    /// assert_eq!(os_path.to_pathbuf(), std::path::PathBuf::from("/foo/bar/baz.txt"));
    /// ```
    pub fn to_pathbuf(&self) -> PathBuf {
        let path = self.to_path();
        path.to_owned()
    }

    /// Returns the path as a Path.
    /// ```rust
    /// use os_path::OsPath;
    ///
    /// let os_path = OsPath::from("/foo/bar/baz.txt");
    /// assert_eq!(os_path.to_path(), std::path::Path::new("/foo/bar/baz.txt"));
    /// ```
    pub fn to_path(&self) -> &Path {
        self.path.as_path()
    }
}

impl OsPath {
    fn build_self<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_string_lossy().to_string();
        let absolute = path.starts_with(ROOT) || path.starts_with(BS) || path.starts_with(FS);
        let directory = path.ends_with(SLASH) || path.ends_with(UP);
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
        if second.components.is_empty() {
            return;
        }
        if first.components.is_empty() && !first.absolute {
            *first = second;
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

impl fmt::Display for OsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_string())
    }
}

impl Serialize for OsPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.build_string())
    }
}

struct OsPathVisitor;

impl<'de> Visitor<'de> for OsPathVisitor {
    type Value = OsPath;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a str or String")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OsPath::from(value))
    }
}

impl<'de> Deserialize<'de> for OsPath {
    fn deserialize<D>(deserializer: D) -> Result<OsPath, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(OsPathVisitor)
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
        Self::build_self(s)
    }
}

impl From<PathBuf> for OsPath {
    fn from(p: PathBuf) -> Self {
        Self::build_self(p)
    }
}

impl From<&PathBuf> for OsPath {
    fn from(p: &PathBuf) -> Self {
        Self::build_self(p)
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
