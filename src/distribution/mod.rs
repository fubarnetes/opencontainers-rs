mod auth;
use auth::{Authenticate, Credential};

use crate::image::Image;

use reqwest::{Client, StatusCode};
use ttl_cache::TtlCache;

#[derive(Debug, Fail)]
#[allow(clippy::large_enum_variant)]
pub enum RegistryError {
    #[fail(display = "Request Error: {:?}", _0)]
    ReqwestError(#[cause] reqwest::Error),

    #[fail(display = "Invalid authentication challenge: {}", _0)]
    InvalidAuthenticationChallenge(String),

    #[fail(display = "Could not get token: {}", _0)]
    CouldNotGetToken(StatusCode),

    #[fail(display = "Could not authenticate")]
    CouldNotAuthenticate,

    #[fail(display = "Manifest Error: {:?}", _0)]
    ManifestError(#[cause] crate::image::manifest::ManifestError),

    #[fail(display = "Unsupported Manifest Schema: {:?}", _0)]
    UnsupportedManifestSchema(crate::image::manifest::ManifestV2Schema),

    #[fail(display = "Image Spec Error: {:?}", _0)]
    ImageSpecError(#[cause] crate::image::spec::ImageSpecError),
}

/// Represents a Registry implementing the [OpenContainer Distribution
/// Spec](https://github.com/opencontainers/distribution-spec/blob/master/spec.md)
pub struct Registry {
    pub url: String,
    client: Client,
    credential_cache: TtlCache<String, Credential>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Registry {{ url: {}, client: {:?} }}",
            self.url, self.client
        )
    }
}

impl Registry {
    /// Create a new registry interface given the URL to a registry.
    ///
    /// Note: The URL should **not** contain a trailing slash.
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    /// let registry = Registry::new("https://registry-1.docker.io");
    ///# assert_eq!(registry.url, "https://registry-1.docker.io");
    /// ```
    ///
    /// # Panics
    /// This function can panic if the backing
    /// [ClientBuilder](https://docs.rs/reqwest/0/reqwest/struct.ClientBuilder.html)
    /// cannot be initialized. This can happen if the native TLS backend
    /// cannot be initialized.
    pub fn new(url: &str) -> Self {
        let client = Client::builder()
            .gzip(true)
            .build()
            .expect("Could not build request client");

        let credential_cache: TtlCache<String, Credential> = TtlCache::new(32);

        Registry {
            url: url.into(),
            client,
            credential_cache,
        }
    }

    fn try_auth(
        &self,
        authenticate: &reqwest::header::HeaderValue,
    ) -> Result<Vec<Credential>, RegistryError> {
        auth::do_challenge(&self.client, authenticate)
    }

    fn attempt_request(
        &self,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
        cred: Option<&Credential>,
    ) -> Result<Result<reqwest::Response, reqwest::Response>, RegistryError> {
        let mut request = self.client.get(url);

        if let Some(headers) = headers {
            request = request.headers(headers.clone());
        }

        if let Some(credential) = cred {
            request = request.authenticate(&credential);
        } else {
            info!("Attempting unauthenticated request");
        }

        let response = request.send().map_err(RegistryError::ReqwestError)?;

        let status = response.status();

        info!("got response: {:?}", response);

        if status.is_success() {
            return Ok(Ok(response));
        }

        Ok(Err(response))
    }

    /// Perform a GET request on the Registry, handling authentication.
    ///
    /// # Authentication
    /// Authentication is handled transiently according to the [Docker
    /// Registry Token Authentication
    /// Specification](https://docs.docker.com/registry/spec/auth/token/)
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let endpoint = format!("{}/v2/", registry.url);
    /// let response = registry.get(endpoint.as_str(), None)
    ///     .expect("Could not perform API Version Check");
    /// assert!(response.status().is_success());
    /// ```
    pub fn get(
        &self,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response, RegistryError> {
        // Try to use the credential if it is cached
        let credential = self.credential_cache.get(url);

        // Attempt request
        let response = match self.attempt_request(url, headers, credential)? {
            Ok(response) => return Ok(response),
            Err(response) => response,
        };

        // Unauthorized
        let unauthorized = response.status() == StatusCode::UNAUTHORIZED;
        let has_authenticate = response
            .headers()
            .contains_key(reqwest::header::WWW_AUTHENTICATE);

        if unauthorized && !has_authenticate {
            return Err(RegistryError::InvalidAuthenticationChallenge(
                "No authentication challenge presented".into(),
            ));
        } else if !unauthorized {
            return Err(RegistryError::CouldNotGetToken(response.status()));
        }

        info!("Authentication required");
        #[allow(clippy::or_fun_call)]
        let authenticate = response
            .headers()
            .get(reqwest::header::WWW_AUTHENTICATE)
            .ok_or(RegistryError::InvalidAuthenticationChallenge(
                "Missing WWW-Authenticate Header".into(),
            ))?;

        let credentials = self.try_auth(authenticate)?;

        // Attempt with each credential we got
        for credential in credentials {
            if let Ok(response) = self.attempt_request(url, headers, Some(&credential))? {
                info!("Got response: {:?}", response);

                // TODO: Cache credential.
                return Ok(response);
            }
        }

        Err(RegistryError::CouldNotAuthenticate)
    }

    /// Create an image handle for a given image
    ///
    /// The type parameter has a trait bound on [image::ImageSelector], which can
    /// be implemented to select which image to use when pulling from a
    /// fat manifest.
    /// For most cases the [image::ImagePlatformSelector] should do just fine.
    ///
    /// # Example
    /// ```
    ///# extern crate opencontainers;
    ///# use opencontainers::Registry;
    ///# use opencontainers::image::TestImageSelector as ImagePlatformSelector;
    ///# let registry = Registry::new("https://registry-1.docker.io");
    /// let manifest = registry.image::<ImagePlatformSelector>("library/hello-world", "latest")
    ///     .expect("Could not get image");
    /// ```
    pub fn image<IS>(&self, name: &str, reference: &str) -> Result<Image, RegistryError>
    where
        IS: crate::image::ImageSelector,
    {
        Image::new::<IS>(self, name, reference)
    }
}
