use std::str::FromStr;

#[derive(Debug, Fail)]
pub enum ManifestError {
    #[fail(display = "JSON Error: {:?}", _0)]
    JsonError(serde_json::Error),

    #[fail(display = "Invalid Schema Version: {}", _0)]
    InvalidSchemaVersion(u64),

    #[fail(display = "Invalid (unknown) Media Type: {}", _0)]
    InvalidMediaType(String),
}

/// Helper struct to determine Image Manifest Schema.
#[derive(Debug, Deserialize)]
struct ManifestSchemaOnlyV2 {
    #[serde(rename = "schemaVersion")]
    schema: u64,
}

impl ManifestSchemaOnlyV2 {
    // Return the schema version.
    pub fn schema(&self) -> u64 {
        self.schema
    }
}

#[derive(Debug, Deserialize)]
// Helper struct to determine Schema 2 Image Manifest media type
struct ManifestMediaTypeOnlyV2_2 {
    /// The MIME type of the referenced object. This should generally be
    /// `application/vnd.docker.container.image.v1+json`.
    #[serde(rename = "mediaType")]
    media_type: String,
}

impl ManifestMediaTypeOnlyV2_2 {
    // Return the schema version.
    pub fn media_type(&self) -> &str {
        &self.media_type
    }
}

/// Enum of Manifest structs for each schema version.
#[derive(Debug)]
pub enum ManifestV2 {
    Schema1(ManifestV2_1),
    Schema2(ManifestV2_2),
    Schema2List(ManifestListV2_2),
}

impl FromStr for ManifestV2 {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match probe_manifest_v2_schema(s)? {
            ManifestV2Schema::Schema1 => serde_json::from_str(s).map(ManifestV2::Schema1),
            ManifestV2Schema::Schema2 => serde_json::from_str(s).map(ManifestV2::Schema2),
            ManifestV2Schema::Schema2List => serde_json::from_str(s).map(ManifestV2::Schema2List),
        }
        .map_err(ManifestError::JsonError)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Discriminants for ManifestV2
pub enum ManifestV2Schema {
    Schema1,
    Schema2,
    Schema2List,
}

impl From<ManifestV2> for ManifestV2Schema {
    fn from(manifest: ManifestV2) -> Self {
        match manifest {
            ManifestV2::Schema1(_) => ManifestV2Schema::Schema1,
            ManifestV2::Schema2(_) => ManifestV2Schema::Schema2,
            ManifestV2::Schema2List(_) => ManifestV2Schema::Schema2List,
        }
    }
}

pub fn probe_manifest_v2_schema(data: &str) -> Result<ManifestV2Schema, ManifestError> {
    let manifest: ManifestSchemaOnlyV2 =
        serde_json::from_str(data).map_err(ManifestError::JsonError)?;

    match manifest.schema() {
        1 => return Ok(ManifestV2Schema::Schema1),
        2 => {}
        schema => return Err(ManifestError::InvalidSchemaVersion(schema)),
    };

    let manifest: ManifestMediaTypeOnlyV2_2 =
        serde_json::from_str(data).map_err(ManifestError::JsonError)?;

    let media_type = manifest.media_type();

    #[allow(clippy::or_fun_call)]
    let media_type_split = media_type
        .split('+')
        .next()
        .ok_or(ManifestError::InvalidMediaType(media_type.into()))?;

    match media_type_split {
        "application/vnd.oci.distribution.manifest.v2" => Ok(ManifestV2Schema::Schema2),
        "application/vnd.oci.distribution.manifest.list.v2" => Ok(ManifestV2Schema::Schema2List),
        // Docker seems to be compatible to OCI, so we also support those.
        "application/vnd.docker.distribution.manifest.v2" => Ok(ManifestV2Schema::Schema2),
        "application/vnd.docker.distribution.manifest.list.v2" => Ok(ManifestV2Schema::Schema2List),
        _ => Err(ManifestError::InvalidMediaType(media_type.into())),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FsLayerV2_1 {
    #[serde(rename = "blobSum")]
    inner: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct V1Compatibility {
    #[serde(rename = "v1Compatibility")]
    inner: String,
}

/// Image Manifest Version 2, Schema 1
#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestV2_1 {
    #[serde(rename = "schemaVersion")]
    schema: u64,

    name: String,
    tag: String,
    architecture: String,

    #[serde(rename = "fsLayers")]
    layers: Vec<FsLayerV2_1>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigV2_2 {
    /// The MIME type of the referenced object. This should generally be
    /// `application/vnd.docker.container.image.v1+json`.
    #[serde(rename = "mediaType")]
    media_type: String,

    /// The size in bytes of the object.
    ///
    /// This field exists so that a client will have an expected size for the
    /// content before validating. If the length of the retrieved content does
    /// not match the specified length, the content should not be trusted.
    size: usize,

    /// The digest of the content, as defined by the [Registry V2 HTTP API
    /// Specificiation](https://docs.docker.com/registry/spec/api/#digest-parameter).
    digest: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LayerV2_2 {
    /// The MIME type of the referenced object.
    ///
    /// This should generally be
    /// `application/vnd.docker.image.rootfs.diff.tar.gzip`. Layers of type
    /// `application/vnd.docker.image.rootfs.foreign.diff.tar.gzip` may be
    /// pulled from a remote location but they should never be pushed.
    #[serde(rename = "mediaType")]
    media_type: String,

    /// The size in bytes of the object
    ///
    /// This field exists so that a client will have an expected size for the
    /// content before validating. If the length of the retrieved content does
    /// not match the specified length, the content should not be trusted.
    size: usize,

    /// The digest of the content, as defined by the [Registry V2 HTTP API
    /// Specificiation](https://docs.docker.com/registry/spec/api/#digest-parameter).
    digest: String,

    /// Provides a list of URLs from which the content may be fetched.
    ///
    /// Content should be verified against the digest and size. This field is
    /// optional and uncommon.
    urls: Option<Vec<String>>,
}

/// Image Manifest Version 2, Schema 2
#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestV2_2 {
    /// This field specifies the image manifest schema version as an integer.
    ///
    /// This schema uses version 2.
    #[serde(rename = "schemaVersion")]
    pub schema: u64,

    /// The MIME type of the manifest. This should be set to
    /// `application/vnd.docker.distribution.manifest.v2+json`.
    #[serde(rename = "mediaType")]
    pub media_type: String,

    /// The config field references a configuration object for a container, by
    /// digest.
    ///
    /// This configuration item is a JSON blob that the runtime uses to
    /// set up the container. This new schema uses a tweaked version of this
    /// configuration to allow image content-addressability on the daemon side.
    #[serde(rename = "config")]
    pub config: ConfigV2_2,

    /// The layer list is ordered starting from the base image
    ///
    /// (opposite order of schema1).
    pub layers: Vec<LayerV2_2>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestPlatformV2_2 {
    /// The architecture field specifies the CPU architecture, for example
    /// amd64 or ppc64le.
    architecture: String,

    /// The os field specifies the operating system, for example linux or
    /// windows.
    os: String,

    /// The optional os.version field specifies the operating system version,
    /// for example 10.0.10586.
    #[serde(rename = "os.version")]
    osversion: Option<String>,

    /// The optional os.features field specifies an array of strings, each
    /// listing a required OS feature (for example on Windows win32k).
    #[serde(rename = "os.features")]
    osfeatures: Option<Vec<String>>,

    /// The optional variant field specifies a variant of the CPU, for example
    /// armv6l to specify a particular CPU variant of the ARM CPU.
    variant: Option<String>,

    /// The optional features field specifies an array of strings, each listing
    /// a required CPU feature (for example sse4 or aes).
    features: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestListEntryV2_2 {
    /// The MIME type of the referenced object.
    ///
    /// This will generally be `application/vnd.docker.image.manifest.v2+json`,
    /// but it could also be `application/vnd.docker.image.manifest.v1+json`
    /// if the manifest list references a legacy schema-1 manifest.
    #[serde(rename = "mediaType")]
    media_type: String,

    /// The size in bytes of the object
    ///
    /// This field exists so that a client will have an expected size for the
    /// content before validating. If the length of the retrieved content does
    /// not match the specified length, the content should not be trusted.
    size: usize,

    /// The digest of the content, as defined by the [Registry V2 HTTP API
    /// Specificiation](https://docs.docker.com/registry/spec/api/#digest-parameter).
    digest: String,

    /// The platform object describes the platform which the image in the
    /// manifest runs on. A full list of valid operating system and architecture
    /// values are listed in the Go language documentation for $GOOS and $GOARCH
    platform: ManifestPlatformV2_2,
}

/// Manifest List
///
/// The manifest list is the “fat manifest” which points to specific image
/// manifests for one or more platforms. Its use is optional, and relatively
/// few images will use one of these manifests.
///
/// A client will distinguish a manifest list from an image manifest based on
/// the Content-Type returned in the HTTP response.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestListV2_2 {
    /// This field specifies the image manifest schema version as an integer.
    ///
    /// This schema uses version 2.
    #[serde(rename = "schemaVersion")]
    pub schema: u64,

    /// The MIME type of the manifest list. This should be set to
    /// `application/vnd.docker.distribution.manifest.list.v2+json`.
    media_type: String,

    /// The manifests field contains a list of manifests for specific platforms.
    manifests: Vec<ManifestListEntryV2_2>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_manifest_v1() {
        let test_data = include_str!("test/manifest-v2-1.test.json");

        let manifest: ManifestV2_1 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(manifest.schema, 1);
        assert_eq!(manifest.name, "hello-world");
        assert_eq!(manifest.tag, "latest");
        assert_eq!(manifest.architecture, "amd64");
        assert_eq!(manifest.layers.len(), 4);
    }

    #[test]
    fn test_manifest_v2() {
        let test_data = include_str!("test/manifest-v2-2.test.json");

        let manifest: ManifestV2_2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(manifest.schema, 2);
        assert_eq!(
            manifest.media_type,
            "application/vnd.docker.distribution.manifest.v2+json"
        );

        assert_eq!(
            manifest.config.media_type,
            "application/vnd.docker.container.image.v1+json"
        );
        assert_eq!(manifest.config.size, 7023);
        assert_eq!(
            manifest.config.digest,
            "sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7"
        );

        assert_eq!(manifest.layers.len(), 3);

        assert_eq!(
            manifest.layers[0],
            LayerV2_2 {
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".into(),
                size: 32654,
                digest: "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f"
                    .into(),
                urls: None,
            }
        );

        assert_eq!(
            manifest.layers[1],
            LayerV2_2 {
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".into(),
                size: 16724,
                digest: "sha256:3c3a4604a545cdc127456d94e421cd355bca5b528f4a9c1905b15da2eb4a4c6b"
                    .into(),
                urls: None,
            }
        );

        assert_eq!(
            manifest.layers[2],
            LayerV2_2 {
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".into(),
                size: 73109,
                digest: "sha256:ec4b8955958665577945c89419d1af06b5f7636b4ac3da7f12184802ad867736"
                    .into(),
                urls: None,
            }
        );
    }

    #[test]
    fn test_manifest_list_v2() {
        let test_data = include_str!("test/manifest-list-v2-2.test.json");

        let manifest_list: ManifestListV2_2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest list");

        assert_eq!(manifest_list.schema, 2);
        assert_eq!(
            manifest_list.media_type,
            "application/vnd.docker.distribution.manifest.list.v2+json"
        );
        assert_eq!(manifest_list.manifests.len(), 2);
    }

    #[test]
    fn test_manifest_schemaonly_schema1() {
        let test_data = include_str!("test/manifest-v2-1.test.json");

        let manifest: ManifestSchemaOnlyV2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(manifest.schema(), 1);
    }

    #[test]
    fn test_manifest_schemaonly_schema2() {
        let test_data = include_str!("test/manifest-v2-2.test.json");

        let manifest: ManifestSchemaOnlyV2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(manifest.schema(), 2);
    }

    #[test]
    fn test_manifest_schemaonly_schema2_list() {
        let test_data = include_str!("test/manifest-list-v2-2.test.json");

        let manifest: ManifestSchemaOnlyV2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(manifest.schema(), 2);
    }

    #[test]
    fn test_manifest_mediatypeonly_schema2() {
        let test_data = include_str!("test/manifest-v2-2.test.json");

        let manifest: ManifestMediaTypeOnlyV2_2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(
            manifest.media_type(),
            "application/vnd.docker.distribution.manifest.v2+json"
        );
    }

    #[test]
    fn test_manifest_mediatypeonly_schema2_list() {
        let test_data = include_str!("test/manifest-list-v2-2.test.json");

        let manifest: ManifestMediaTypeOnlyV2_2 =
            serde_json::from_str(test_data).expect("Could not deserialize manifest");

        assert_eq!(
            manifest.media_type(),
            "application/vnd.docker.distribution.manifest.list.v2+json"
        );
    }

    #[test]
    fn test_probe_manifest_schema1() {
        let test_data = include_str!("test/manifest-v2-1.test.json");
        let schema = probe_manifest_v2_schema(test_data).expect("could not probe manifest schema");

        assert_eq!(schema, ManifestV2Schema::Schema1);
    }

    #[test]
    fn test_probe_manifest_schema2() {
        let test_data = include_str!("test/manifest-v2-2.test.json");
        let schema = probe_manifest_v2_schema(test_data).expect("could not probe manifest schema");

        assert_eq!(schema, ManifestV2Schema::Schema2);
    }

    #[test]
    fn test_probe_manifest_schema2_list() {
        let test_data = include_str!("test/manifest-list-v2-2.test.json");
        let schema = probe_manifest_v2_schema(test_data).expect("could not probe manifest schema");

        assert_eq!(schema, ManifestV2Schema::Schema2List);
    }

    #[test]
    fn test_parse_manifest_v2() {
        let test_data = include_str!("test/manifest-v2-1.test.json");
        let manifest: ManifestV2 = test_data
            .parse()
            .expect("Could not parse manifest schema 1");
        assert_eq!(ManifestV2Schema::from(manifest), ManifestV2Schema::Schema1);

        let test_data = include_str!("test/manifest-v2-2.test.json");
        let manifest: ManifestV2 = test_data
            .parse()
            .expect("Could not parse manifest schema 2");
        assert_eq!(ManifestV2Schema::from(manifest), ManifestV2Schema::Schema2);

        let test_data = include_str!("test/manifest-list-v2-2.test.json");
        let manifest: ManifestV2 = test_data
            .parse()
            .expect("Could not parse manifest schema 2 list");
        assert_eq!(
            ManifestV2Schema::from(manifest),
            ManifestV2Schema::Schema2List
        );
    }

}
