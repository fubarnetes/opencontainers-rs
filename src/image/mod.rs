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
    /// let image = opencontainers::Image::new(&registry, "library/hello-world", "latest")
    ///     .expect("Could not get image");
    /// ```
    pub fn new(registry: &'a Registry, name: &str, reference: &str) -> Result<Self, RegistryError> {
        let url = format!("{}/v2/{}/manifests/{}", registry.url, name, reference);

        // Make sure we only accept schema 2, if we don't set this, we will get
        // schema1 by default.
        // For now, do not support Manifest Lists.
        let accept_types = vec![
            "application/vnd.oci.distribution.manifest.v2+json",
            "application/vnd.docker.distribution.manifest.v2+json",
        ];

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            accept_types.join(",").parse().unwrap(),
        );

        let manifest = registry
            .get(&url, Some(&headers))?
            .text()
            .map_err(RegistryError::ReqwestError)?
            .parse()
            .map_err(RegistryError::ManifestError)?;

        Ok(Self { registry, manifest })
    }

    /// Return an image manifest
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let manifest = registry.image("library/hello-world", "latest")
    ///     .expect("Could not get image")
    ///     .manifest();
    /// ```
    pub fn manifest(&self) -> &ManifestV2 {
        &self.manifest
    }
}
