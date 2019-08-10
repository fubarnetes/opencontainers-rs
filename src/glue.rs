//! Module containing traits that glue together code from the runtime, image and distribution specs.
//!
//! Mostly contains traits that need to be implemented by a consumer of this library, as well as
//! some simplistic implementations.

use crate::image::Image;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[derive(Debug, Fail)]
pub enum UnpackError {
    #[fail(display = "Could not read tar entries: {}", _0)]
    GetEntries(std::io::Error),

    #[fail(display = "Could not get entry: {}", _0)]
    GetEntry(std::io::Error),

    #[fail(display = "Could not get entry path: {}", _0)]
    GetEntryPath(std::io::Error),

    #[fail(display = "Could not unpack entry: {}", _0)]
    UnpackEntry(std::io::Error),

    #[fail(display = "Cannot extract files outside of root filesystem: {:?}", _0)]
    AttemptedFilesystemTraversal(std::path::PathBuf),
}

/// Return th path to the file to be deleted, if it is a whiteout file, otherwise None.
///
/// # Examples
/// ```
///# use opencontainers::glue::get_whiteout_path;
///# use std::path::PathBuf;
/// let whiteout = PathBuf::from("a/b/.wh.c");
/// let to_delete = get_whiteout_path(whiteout).unwrap();
/// assert_eq!(to_delete, PathBuf::from("a/b/c"));
/// ```
///
/// If the file is not a whiteout, `None` is returned.AsMut
/// ```
///# use opencontainers::glue::get_whiteout_path;
///# use std::path::PathBuf;
/// let path = PathBuf::from("a/b/c");
/// assert!(get_whiteout_path(path).is_none());
/// ```
pub fn get_whiteout_path<P: AsRef<std::path::Path>>(path: P) -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    return get_whiteout_path_unix(path);

    #[cfg(windows)]
    return get_whiteout_path_windows(path);
}

#[cfg(unix)]
fn get_whiteout_path_unix<P: AsRef<std::path::Path>>(path: P) -> Option<std::path::PathBuf> {
    let filename = path.as_ref().file_name()?;

    let bytes: Vec<u8> = filename.as_bytes().into();
    if !bytes.starts_with(".wh.".as_bytes()) {
        return None;
    }

    let mut path = path.as_ref().to_owned();
    path.set_file_name(std::ffi::OsStr::from_bytes(&bytes[4..]));
    return Some(path);
}

#[cfg(windows)]
fn get_whiteout_path_windows<P: AsRef<std::path::Path>>(path: P) -> Option<std::path::PathBuf> {
    let filename = path.as_ref().file_name()?;

    let wide: Vec<u16> = filename.encode_wide().collect();
    let needle: std::ffi::OsString = ".wh.".to_owned().into();
    if !wide.starts_with(&needle.encode_wide().collect::<Vec<u16>>()) {
        return None;
    }

    let mut path = path.as_ref().to_owned();
    path.set_file_name(std::ffi::OsString::from_wide(&wide[4..]));
    return Some(path);
}
/// A trait that describes the actions required to create a container's root filesystem
/// from an image.
pub trait Unpack {
    /// Called in order for each layer in the image
    fn apply<R: std::io::Read>(&self, layer: tar::Archive<R>) -> Result<(), UnpackError>;
    fn apply_change<R: std::io::Read>(&self, entry: tar::Entry<R>) -> Result<(), UnpackError>;

    fn unpack(&self, image: &Image) -> Result<(), crate::Error> {
        for layer in image
            .manifest()
            .layers()
            .map_err(crate::Error::RegistryError)?
        {
            let tar = image
                .get_layer(layer)
                .map_err(crate::Error::RegistryError)?;
            self.apply(tar).map_err(crate::Error::UnpackError)?;
        }

        Ok(())
    }
}

/// Extracts an image to a folder.
#[derive(Debug, Clone)]
pub struct SimpleFolderUnpacker {
    /// Path to the folder the image will be extracted to.
    root: std::path::PathBuf,
}

impl SimpleFolderUnpacker {
    /// Construct a new SimpleFolderUnpacker, given an existing path.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<std::path::PathBuf>,
    {
        Self { root: path.into() }
    }
}

impl Unpack for SimpleFolderUnpacker {
    fn apply_change<R: std::io::Read>(&self, mut entry: tar::Entry<R>) -> Result<(), UnpackError> {
        let path: std::path::PathBuf = entry.path().map_err(UnpackError::GetEntryPath)?.into();
        if let Some(filename) = path.file_name() {
            if filename == std::ffi::OsStr::new(".wh..wh..opq") {
                // TODO: Whiteout the complete directory including all siblings.
                return Ok(());
            }

            if filename.to_string_lossy().starts_with(".wh.") {}
        }

        // Apply addition or modification:
        // Additions and Modifications are represented the same in the changeset tar archive.
        if !entry
            .unpack_in(self.root.as_path())
            .map_err(UnpackError::UnpackEntry)?
        {
            return Err(UnpackError::AttemptedFilesystemTraversal(path.into()));
        }

        Ok(())
    }

    fn apply<R: std::io::Read>(&self, mut layer: tar::Archive<R>) -> Result<(), UnpackError> {
        let entries = layer.entries().map_err(UnpackError::GetEntries)?;

        for entry in entries {
            let entry = entry.map_err(UnpackError::GetEntry)?;
            self.apply_change(entry)?;
        }

        Ok(())
    }
}
