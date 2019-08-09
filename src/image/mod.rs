use crate::distribution::{Registry, RegistryError};
mod go;

pub mod manifest;
pub mod spec;
use manifest::Digest;
pub use manifest::ManifestV2;

#[derive(Debug)]
pub struct Image<'a> {
    registry: &'a Registry,
    name: String,
    manifest: ManifestV2,
}

/// Trait to determine which image to select from a Manifest.
pub trait ImageSelector {
    /// Select a specific ManifestV2Entry from a Manifest
    fn select_manifest<'a>(
        manifest_list: &'a manifest::ManifestListV2_2,
    ) -> Option<&'a manifest::ManifestListEntryV2_2>;
}

/// Select the best image based on the current platform.
pub struct ImagePlatformSelector {}

impl ImageSelector for ImagePlatformSelector {
    fn select_manifest<'a>(
        manifest_list: &'a manifest::ManifestListV2_2,
    ) -> Option<&'a manifest::ManifestListEntryV2_2> {
        manifest_list
            .manifests
            .iter()
            .filter(|m| m.platform.current_platform_matches())
            .next()
    }
}

/// Utility image selector for tests, always takes the first available image manifest.
pub struct TestImageSelector {}

impl ImageSelector for TestImageSelector {
    fn select_manifest<'a>(
        manifest_list: &'a manifest::ManifestListV2_2,
    ) -> Option<&'a manifest::ManifestListEntryV2_2> {
        manifest_list
            .manifests
            .iter()
            .next()
    }
}

impl<'a> Image<'a> {
    /// Create a new image given a specific repository
    ///
    /// Consider using [Registry::image] instead.
    ///
    /// The type parameter has a trait bound on [ImageSelector], which can
    /// be implemented to select which image to use when pulling from a
    /// fat manifest.
    /// For most cases the [ImagePlatformSelector] should do just fine.
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# use opencontainers::image::TestImageSelector as ImagePlatformSelector;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let image = opencontainers::Image::new::<ImagePlatformSelector>(&registry, "library/hello-world", "latest")
    ///     .expect("Could not get image");
    /// ```
    pub fn new<IS>(
        registry: &'a Registry,
        name: &str,
        reference: &str,
    ) -> Result<Self, RegistryError>
    where
        IS: ImageSelector,
    {
        let name = name.to_owned();

        let url = format!("{}/v2/{}/manifests/{}", registry.url, name, reference);

        // Make sure we only accept schema 2, if we don't set this, we will get
        // schema1 by default.
        // For now, do not support Manifest Lists.
        let accept_types = vec![
            "application/vnd.oci.distribution.manifest.list.v2+json",
            "application/vnd.oci.distribution.manifest.v2+json",
            "application/vnd.docker.distribution.manifest.list.v2+json",
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

        let mut image = Self {
            registry,
            name,
            manifest,
        };

        match image.manifest {
            ManifestV2::Schema2List(ref l) => {
                image.manifest =
                    ManifestV2::Schema2(l.get_current_platform_manifest::<IS>(&image)?);
            }
            _ => {}
        };

        Ok(image)
    }

    /// Return an image manifest
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# use opencontainers::image::TestImageSelector as ImagePlatformSelector;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let manifest = registry.image::<ImagePlatformSelector>("library/hello-world", "latest")
    ///     .expect("Could not get image")
    ///     .manifest();
    /// ```
    pub fn manifest(&self) -> &ManifestV2 {
        &self.manifest
    }

    pub fn get_blob(&self, digest: &Digest) -> Result<reqwest::Response, RegistryError> {
        let url = format!("{}/v2/{}/blobs/{}", self.registry.url, self.name, digest);

        self.registry.get(&url, None)
    }

    /// Return the image runtime configuration
    pub fn config(&self) -> Result<spec::ImageV1, RegistryError> {
        match manifest::ManifestV2Schema::from(self.manifest()) {
            manifest::ManifestV2Schema::Schema2 => {}
            other => return Err(RegistryError::UnsupportedManifestSchema(other)),
        };

        let config_digest = match self.manifest() {
            ManifestV2::Schema2(m) => m.config.digest(),
            _ => unreachable!(),
        };

        self.get_blob(config_digest)?
            .text()
            .map_err(RegistryError::ReqwestError)?
            .parse()
            .map_err(RegistryError::ImageSpecError)
    }

    /// Get a layer, decompressing if necessary
    pub fn get_layer<L>(
        &self,
        layer: &L,
    ) -> Result<tar::Archive<Box<dyn std::io::Read>>, RegistryError>
    where
        L: crate::image::manifest::Layer + ?Sized,
    {
        let response = self.get_blob(layer.digest())?;

        if let Some(media_type) = layer.media_type() {
            if !media_type.is_gzipped() {
                // No need to wrap reader
                return Ok(tar::Archive::new(Box::new(response)));
            }
        }

        // Otherwise, wrap in a flate2::read::GzDecoder
        let decoder = flate2::read::GzDecoder::new(response);
        Ok(tar::Archive::new(Box::new(decoder)))
    }
}
