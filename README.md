
# OsPath
The way paths should be handled.

## Description
Provides interoperability with std::path::PathBuf and std::path::Path, while handling paths intelligently and not as mindless strings.

### Key Features

#### Path Normalization
Paths are always normalized to the platform's native path format.

#### False Root Handling
False root errors occur when you have a path, lets say `/foo/bar` and you try to `join()` or `push()`  `/baz.txt` to it. With the standard libraries Path and PathBuf, you'll end up with `/baz.txt` as your path. This is very counter intuitive, and not what most users expect, and is not user friendly at all, as you are forced to write extra code to strip slashes from the start of your paths.

Instead, OsPath will do what you expect, and return `/foo/bar/baz.txt`.

And OsPath does this while still assuming at the start that both paths were absolute. If you queried either path beforehand, they would both return true for `is_absolute()`. However, when you joined the two paths, OsPath will provide the expected behavior

Note that this is not a problem on Windows, as attempting to join any path starting with C:\ is nonsensical, while joinging a path prefixed with `/` or `\\` is not.

#### Path Traversal
Yes, if you `join()` or `push()` a path that starts with `..`, OsPath will traverse the path, and build the correct path. `/foo/bar/baz/` joined with `../pow.txt` will return `/foo/bar/pow.txt`.

OsPath can handle multiple `..` in a row, and will traverse the path correctly. `/foo/bar/baz/` joined with `../../pow.txt` will return `/foo/pow.txt`.

And, if your path ends in a file, and you `join()` or `push()` a path that starts with `..`, OsPath will traverse the path, and build the correct path. `/foo/bar/baz.txt` joined with `../pow.txt` will return `/foo/pow.txt`.

> Note: Path traversal is not automatic when using the `OsPath::from()` method, because it can be advantageous to retain the `..` in the path. For example, if you are passing this path to an outside program, you may want to retain the `..` so that the program can handle the path traversal itself. However, it can be invoked with `os_path.resolve()`.

#### File And Directory Handling
If the path ends in a `/` or `\\` OsPath assumes this is a directory, otherwise it's a file.

## Usage
Use as you would std::path::PathBuf.

It can be passed into any function that takes <P: AsRef<Path>>(path: P) as an argument, and can be built from the same, so it is fully interoperable with the standard library.

## License
MIT License

## Project status
Very stable on Unix systems. Documentation is mostly complete.

I'm adding implementations as needs arise.

I'm aware that there is some odd behavior with Windows path handling, sometimes you get a "C:C:\\" prefix.

I'm working on a fix, as currently OsPath will not pass all the Windows specific unit tests. When that happens, I'll move to version 1.0.0


