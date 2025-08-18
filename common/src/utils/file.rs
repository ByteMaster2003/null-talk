//! File utilities for reading and writing files.
//! Provides functions to read files, as well as resolve paths.

use std::{
    env, fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

/// Reads the contents of a file and returns it as a String.
/// Returns an error if the file cannot be read.
pub fn read_file_contents(path: &PathBuf) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|e| Error::new(ErrorKind::Other, e))
}

/// Expands a tilde ("~") in a file path to the user's home directory.
fn expand_tilde<P: AsRef<Path>>(input: P) -> PathBuf {
    let path = input.as_ref();

    // Check if first component is "~"
    if let Some(first) = path.components().next() {
        if first.as_os_str() == "~" {
            if let Some(home) = dirs::home_dir() {
                return home.join(path.strip_prefix("~").unwrap());
            }
        } else {
            return env::current_dir().unwrap().join(path);
        }
    }

    path.to_path_buf()
}

/// Resolves a file path to an absolute path.
pub fn resolve_path<P: AsRef<Path>>(input: P) -> Result<PathBuf, Error> {
    let path = expand_tilde(input.as_ref());
    let path = path
        .canonicalize()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    if path.exists() {
        Ok(path)
    } else {
        Err(Error::new(ErrorKind::NotFound, "File not found"))
    }
}
