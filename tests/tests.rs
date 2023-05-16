use os_path::OsPath;

#[test]
fn test_impossible_path() {
    assert_eq!(
        OsPath::from("/\\///\\foo///bar\\\\baz.txt"),
        OsPath::from("/foo/bar/baz.txt")
    )
}

#[test]
fn test_absolute_path_resolution() {
    #[cfg(unix)]
    {
        assert!(OsPath::from("/foo").absolute());
        assert!(OsPath::from("/foo/").absolute());
        assert!(!OsPath::from("foo/").absolute());
        assert!(!OsPath::from("foo/bar/").absolute());
    }
    #[cfg(windows)]
    {
        assert!(OsPath::from("C:\\foo").absolute());
        assert!(OsPath::from("C:\\foo\\").absolute());
        assert!(!OsPath::from("foo\\").absolute());
        assert!(!OsPath::from("foo\\bar\\").absolute());
        assert!(!OsPath::from("\\foo\\bar\\").absolute());
    }
}

#[test]
fn test_directory_resolution() {
    #[cfg(unix)]
    {
        assert!(OsPath::from("/foo/").is_dir());
        assert!(OsPath::from("/foo").is_file());
        assert!(OsPath::from("/foo/bar.txt").is_file());
    }
    #[cfg(windows)]
    {
        assert!(OsPath::from("C:\\foo\\").is_dir());
        assert!(OsPath::from("C:\\foo").is_file);
        assert!(OsPath::from("C:\\foo\\bar.txt").is_file);
    }
}

#[test]
fn test_traversal_resolution() {
    #[cfg(unix)]
    {
        let mut path = OsPath::from("/foo/bar/baz/../../zip.txt");
        path.resolve();

        assert_eq!(path, OsPath::from("/foo/zip.txt"));
    }
}

#[test]
fn test_directory_traversal() {
    #[cfg(unix)]
    {
        assert_eq!(
            OsPath::from("/foo1/foo2/foo3/bar.txt").join("../baz/zip.txt"),
            OsPath::from("/foo1/foo2/baz/zip.txt")
        );

        assert_eq!(
            OsPath::from("/foo1/foo2/foo3/bar.txt").join("../../baz/zip.txt"),
            OsPath::from("/foo1/baz/zip.txt")
        );

        assert_eq!(
            OsPath::from("/foo1/foo2/foo3/").join("../zip.txt"),
            OsPath::from("/foo1/foo2/zip.txt")
        );
    }

    #[cfg(windows)]
    {
        assert_eq!(
            OsPath::from("C:\\foo1\\foo2\\foo3\\bar.txt").join("..\\..\\baz\\zip.txt"),
            OsPath::from("C:\\foo1\\baz\\zip.txt")
        );

        assert_eq!(
            OsPath::from("C:\\foo1\\foo2\\foo3\\bar.txt").join("..\\..\\..\\baz\\zip.txt"),
            OsPath::from("C:\\baz\\zip.txt")
        );

        assert_eq!(
            OsPath::from("C:\\foo1\\foo2\\foo3\\").join("..\\zip.txt"),
            OsPath::from("C:\\foo1\\foo2\\zip.txt")
        );
    }
}

#[test]
fn test_false_root_protection() {
    #[cfg(unix)]
    {
        assert_eq!(
            OsPath::from("/test/path/").join("/more/path/foo.txt"),
            OsPath::from("/test/path/more/path/foo.txt")
        );
    }

    #[cfg(windows)]
    {
        assert_eq!(
            OsPath::from("C:\\test\\path\\").join("\\more\\path\\foo.txt"),
            OsPath::from("C:\\test\\path\\more\\path\\foo.txt")
        );
    }
}

#[test]
fn test_name() {
    #[cfg(unix)]
    {
        let path = OsPath::from("/foo/bar/baz.txt");
        assert_eq!(path.name(), Some(&"baz.txt".to_string()));

        let path = OsPath::from("/foo/bar/");
        assert_eq!(path.name(), Some(&"bar".to_string()))
    }

    #[cfg(windows)]
    {
        let path = OsPath::from("C:\\foo\\bar\\baz.txt");
        assert_eq!(path.name(), Some(&"baz.txt".to_string()));

        let path = OsPath::from("C:\\foo\\bar\\");
        assert_eq!(path.name(), Some(&"bar".to_string()))
    }
}

#[test]
fn test_parent() {
    #[cfg(unix)]
    {
        let path = OsPath::from("/foo/bar/baz.txt");
        assert_eq!(path.parent(), Some(OsPath::from("/foo/bar")));

        let path = OsPath::from("/foo/bar/");
        assert_eq!(path.parent(), Some(OsPath::from("/foo/")));
    }

    #[cfg(windows)]
    {
        let path = OsPath::from("C:\\foo\\bar\\baz.txt");
        assert_eq!(path.parent(), Some(OsPath::from("C:\\foo\\bar")));

        let path = OsPath::from("C:\\foo\\bar\\");
        assert_eq!(path.parent(), Some(OsPath::from("C:\\foo\\")));
    }
}
