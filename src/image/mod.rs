use crate::distribution::{Registry, RegistryError};

pub mod manifest;
pub use manifest::ManifestV2;

#[derive(Debug)]
pub struct Image<'a> {
    registry: &'a Registry,
    manifest: ManifestV2,
}

impl<'a> Image<'a> {
    /// Create a new image given a specific repository
    ///
    /// Consider using [Registry::image] instead.
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let image = opencontainers::Image::new(&registry, "hello-world", "latest")
    ///     .expect("Could not get image");
    /// ```
    pub fn new(registry: &'a Registry, name: &str, reference: &str) -> Result<Self, RegistryError> {
        let manifest = registry.manifest(name, reference)?;
        Ok(Self { registry, manifest })
    }

    /// Return an image manifest
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let manifest = registry.image("hello-world", "latest")
    ///     .expect("Could not get image")
    ///     .manifest();
    /// ```
    pub fn manifest(&self) -> &ManifestV2 {
        &self.manifest
    }
}
