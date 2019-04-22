pub use super::go::{GoArch, GoOs};
use std::collections::HashMap;

#[derive(Debug, Fail)]
#[allow(clippy::large_enum_variant)]
pub enum ImageSpecError {
    #[fail(display = "JSON Error: {:?}", _0)]
    JsonError(serde_json::Error),
}

/// Image structure.
///
/// # Spec
///
/// > # Image JSON
/// >
/// > * Each image has an associated JSON structure which describes some basic
/// >   information about the image such as date created, author, as well as
/// >   execution/runtime configuration like its entrypoint, default arguments,
/// >   networking, and volumes.
/// > * The JSON structure also references a cryptographic hash of each layer
/// >   used by the image, and provides history information for those layers.
/// > * The JSON structure also references a cryptographic hash of each layer
/// >   used by the image, and provides history information for those layers.
/// > * Changing it means creating a new derived image, instead of changing the
/// >   existing image.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageV1 {
    /// A combinedChanging it means creating a new derived image, instead of changing the existing image.
    /// defined by RFC 3339, section 5.6.
    created: Option<String>,

    /// Gives the name and/or email address of the person or entity which
    /// created and is responsible for maintaining the image.
    author: Option<String>,

    /// The CPU architecture which the binaries in this image are built to run
    /// on. Configurations SHOULD use, and implementations SHOULD understand,
    /// values listed in the Go Language document for GOARCH.
    pub architecture: GoArch,

    /// The name of the operating system which the image is built to run on.
    /// Configurations SHOULD use, and implementations SHOULD understand, values
    /// listed in the Go Language document for GOOS.
    pub os: GoOs,

    /// The execution parameters which SHOULD be used as a base when running a
    /// container using the image. This field can be null, in which case any
    /// execution parameters should be specified at creation of the container.
    config: Option<ConfigV1>,

    /// The rootfs key references the layer content addresses used by the image.
    /// This makes the image config hash depend on the filesystem hash.
    rootfs: RootFSV1,

    /// Describes the history of each layer. The array is ordered from first to
    /// last.
    history: Option<Vec<HistoryV1>>,
}

impl std::str::FromStr for ImageV1 {
    type Err = ImageSpecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(ImageSpecError::JsonError)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Empty {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigV1 {
    /// The username or UID which is a platform-specific structure that allows
    /// specific control over which user the process run as. This acts as a
    /// default value to use when the value is not specified when creating a
    /// container. For Linux based systems, all of the following are valid:
    /// `user`, `uid`, `user:group`, `uid:gid`, `uid:group`, `user:gid`.
    /// If `group`/`gid` is not specified, the default group and supplementary
    /// groups of the given `user`/`uid` in `/etc/passwd` from the container are
    /// applied.
    #[serde(rename = "User")]
    user: Option<String>,

    /// A set of ports to expose from a container running this image. Its keys
    /// can be in the format of: `port/tcp`, `port/udp`, `port` with the default
    /// protocol being tcp if not specified. These values act as defaults and
    /// are merged with any specified when creating a container.
    ///
    /// NOTE: This JSON structure value is unusual because it is a direct JSON
    /// serialization of the Go type `map[string]struct{}` and is represented in
    /// JSON as an object mapping its keys to an empty object.
    #[serde(rename = "ExposedPorts")]
    exposed_ports: Option<HashMap<String, Empty>>,

    /// Entries are in the format of VARNAME=VARVALUE. These values act as
    /// defaults and are merged with any specified when creating a container.
    #[serde(rename = "Env")]
    env: Option<Vec<String>>,

    /// A list of arguments to use as the command to execute when the container
    /// starts. These values act as defaults and may be replaced by an
    /// entrypoint specified when creating a container.
    #[serde(rename = "Entrypoint")]
    entrypoint: Option<Vec<String>>,

    /// Default arguments to the entrypoint of the container. These values act
    /// as defaults and may be replaced by any specified when creating a
    /// container. If an Entrypoint value is not specified, then the first entry
    /// of the `Cmd` array SHOULD be interpreted as the executable to run.
    #[serde(rename = "Cmd")]
    cmd: Option<Vec<String>>,

    /// A set of directories describing where the process is likely write data
    /// pecific to a container instance. NOTE: This JSON structure value is
    /// unusual because it is a direct JSON serialization of the Go type
    /// `map[string]struct{}` and is represented in JSON as an object mapping
    /// its keys to an empty object.
    #[serde(rename = "Volumes")]
    volumes: Option<HashMap<String, Empty>>,

    /// Sets the current working directory of the entrypoint process in the
    /// container. This value acts as a default and may be replaced by a working
    /// directory specified when creating a container.
    #[serde(rename = "WorkingDir")]
    working_dir: Option<String>,

    /// The field contains arbitrary metadata for the container. This property
    /// MUST use the [annotation rules]
    ///
    /// [annotation rules]: https://github.com/opencontainers/image-spec/blob/master/annotations.md#rules.
    #[serde(rename = "Labels")]
    labels: Option<HashMap<String, String>>,

    /// The field contains the system call signal that will be sent to the
    /// container to exit. The signal can be a signal name in the format
    /// SIGNAME, for instance SIGKILL rSIGTMIN+3.
    #[serde(rename = "StopSignal")]
    stop_signal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootFSV1 {
    /// MUST be set to `layers`. Implementations MUST generate an error if they
    /// encounter a unknown value while verifying or unpacking an image.
    r#type: String,

    /// An array of layer content hashes (DiffIDs), in order from first to last.
    diff_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryV1 {
    //// A combined date and time at which the layer was created, formatted as
    /// defined by RFC 3339, section 5.6.
    created: Option<String>,

    /// The author of the build point.
    author: Option<String>,

    /// The command which created the layer.
    created_by: Option<String>,

    /// A custom message set when creating the layer.
    comment: Option<String>,

    /// This field is used to mark if the history item created a filesystem
    /// diff. It is set to true if this history item doesn't correspond to an
    /// actual layer in the rootfs section (for example, Dockerfile's ENV
    /// command results in no change to the filesystem).
    empty_layer: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_config_v1() {
        let test_data = include_str!("test/config-v1.test.json");

        let image: ImageV1 =
            serde_json::from_str(test_data).expect("Could not deserialize configs");

        assert_eq!(image.architecture, GoArch::AMD64);
        assert_eq!(image.os, GoOs::Linux);
    }
}
