use pathz::PathZ;

#[test]
fn test_impossible_path() {
    assert_eq!(
        PathZ::from("/\\///\\foo///bar\\\\baz.txt"),
        PathZ::from("/foo/bar/baz.txt")
    )
}

#[test]
fn test_absolute_path_resolution() {
    #[cfg(unix)]
    {
        assert!(PathZ::from("/foo").absolute());
        assert!(PathZ::from("/foo/").absolute());
        assert!(!PathZ::from("foo/").absolute());
        assert!(!PathZ::from("foo/bar/").absolute());
    }
    #[cfg(windows)]
    {
        assert!(PathZ::from("C:\\foo").absolute());
        assert!(PathZ::from("C:\\foo\\").absolute());
        assert!(!PathZ::from("foo\\").absolute());
        assert!(!PathZ::from("foo\\bar\\").absolute());
        assert!(!PathZ::from("\\foo\\bar\\").absolute());
    }
}

#[test]
fn test_directory_resolution() {
    #[cfg(unix)]
    {
        assert!(PathZ::from("/foo/").is_dir());
        assert!(PathZ::from("/foo").is_file());
        assert!(PathZ::from("/foo/bar.txt").is_file());
    }
    #[cfg(windows)]
    {
        assert!(PathZ::from("C:\\foo\\").is_dir());
        assert!(PathZ::from("C:\\foo").is_file);
        assert!(PathZ::from("C:\\foo\\bar.txt").is_file);
    }
}

#[test]
fn test_traversal_resolution() {
    #[cfg(unix)]
    {
        let mut path = PathZ::from("/foo/bar/baz/../../zip.txt");
        path.resolve();

        assert_eq!(path, PathZ::from("/foo/zip.txt"));
    }
}

#[test]
fn test_directory_traversal() {
    #[cfg(unix)]
    {
        assert_eq!(
            PathZ::from("/foo1/foo2/foo3/bar.txt").join("../baz/zip.txt"),
            PathZ::from("/foo1/foo2/baz/zip.txt")
        );

        assert_eq!(
            PathZ::from("/foo1/foo2/foo3/bar.txt").join("../../baz/zip.txt"),
            PathZ::from("/foo1/baz/zip.txt")
        );

        assert_eq!(
            PathZ::from("/foo1/foo2/foo3/").join("../zip.txt"),
            PathZ::from("/foo1/foo2/zip.txt")
        );
    }

    #[cfg(windows)]
    {
        assert_eq!(
            PathZ::from("C:\\foo1\\foo2\\foo3\\bar.txt").join("..\\..\\baz\\zip.txt"),
            PathZ::from("C:\\foo1\\baz\\zip.txt")
        );

        assert_eq!(
            PathZ::from("C:\\foo1\\foo2\\foo3\\bar.txt").join("..\\..\\..\\baz\\zip.txt"),
            PathZ::from("C:\\baz\\zip.txt")
        );

        assert_eq!(
            PathZ::from("C:\\foo1\\foo2\\foo3\\").join("..\\zip.txt"),
            PathZ::from("C:\\foo1\\foo2\\zip.txt")
        );
    }
}

#[test]
fn test_false_root_protection() {
    #[cfg(unix)]
    {
        assert_eq!(
            PathZ::from("/test/path/").join("/more/path/foo.txt"),
            PathZ::from("/test/path/more/path/foo.txt")
        );
    }

    #[cfg(windows)]
    {
        assert_eq!(
            PathZ::from("C:\\test\\path\\").join("\\more\\path\\foo.txt"),
            PathZ::from("C:\\test\\path\\more\\path\\foo.txt")
        );
    }
}

#[test]
fn test_name() {
    #[cfg(unix)]
    {
        let path = PathZ::from("/foo/bar/baz.txt");
        assert_eq!(path.name(), Some(&"baz.txt".to_string()));

        let path = PathZ::from("/foo/bar/");
        assert_eq!(path.name(), Some(&"bar".to_string()))
    }

    #[cfg(windows)]
    {
        let path = PathZ::from("C:\\foo\\bar\\baz.txt");
        assert_eq!(path.name(), Some(&"baz.txt".to_string()));

        let path = PathZ::from("C:\\foo\\bar\\");
        assert_eq!(path.name(), Some(&"bar".to_string()))
    }
}

#[test]
fn test_parent() {
    #[cfg(unix)]
    {
        let path = PathZ::from("/foo/bar/baz.txt");
        assert_eq!(path.parent(), Some(PathZ::from("/foo/bar")));

        let path = PathZ::from("/foo/bar/");
        assert_eq!(path.parent(), Some(PathZ::from("/foo/")));
    }

    #[cfg(windows)]
    {
        let path = PathZ::from("C:\\foo\\bar\\baz.txt");
        assert_eq!(path.parent(), Some(PathZ::from("C:\\foo\\bar")));

        let path = PathZ::from("C:\\foo\\bar\\");
        assert_eq!(path.parent(), Some(PathZ::from("C:\\foo\\")));
    }
}
