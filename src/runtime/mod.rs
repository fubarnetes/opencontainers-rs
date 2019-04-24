use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod config;
pub use config::Config;

/// Filesystem Bundle
///
/// A set of files organized in a certain way, and containing all the necessary
/// data and metadata for any compliant runtime to perform all standard
/// operations against it. See also [MacOS application bundles] for a similar
/// use of the term bundle.
///
/// #[MacOS application bundles]: https://en.wikipedia.org/wiki/Bundle_%28macOS%29
pub struct Bundle {
    /// The path to the Bundle
    pub path: PathBuf,

    /// Representation of the data in `config.json` in the root of the bundle.
    pub config: Config,
}

/// Enum representing the runtime state of a container
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RuntimeStatus {
    /// The container is being created (step 2 in the lifecycle)
    Creating,

    /// The runtime has finished the create operation (after step 2 in the
    /// lifecycle), and the container process has neither exited nor executed
    /// the user-specified program
    Created,

    /// The container process has executed the user-specified program but has
    /// not exited (after step 5 in the lifecycle)
    Running,

    /// The container process has exited (step 7 in the lifecycle)
    Stopped,

    /// Additional values MAY be defined by the runtime, however, they MUST be
    /// used to represent new runtime states not defined above.
    Other(String),
}

impl std::str::FromStr for RuntimeStatus {
    type Err = void::Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "creating" => Ok(RuntimeStatus::Creating),
            "created" => Ok(RuntimeStatus::Created),
            "running" => Ok(RuntimeStatus::Running),
            "stopped" => Ok(RuntimeStatus::Stopped),
            other => Ok(RuntimeStatus::Other(other.into())),
        }
    }
}

impl std::fmt::Display for RuntimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RuntimeStatus::Creating => "creating",
                RuntimeStatus::Created => "created",
                RuntimeStatus::Running => "running",
                RuntimeStatus::Stopped => "stopped",
                RuntimeStatus::Other(ref s) => s,
            }
        )
    }
}

impl<'de> Deserialize<'de> for RuntimeStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for RuntimeStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    /// The OCI specification version used when creating the container
    #[serde(rename = "ociVersion")]
    oci_version: String,

    /// The container's ID.
    ///
    /// This MUST be unique across all containers on this host. There is no
    /// requirement that it be unique across hosts.
    id: String,

    /// The Runtime State of the container
    status: RuntimeStatus,

    /// The ID of the container process, as seen by the host.
    ///
    /// REQUIRED when [status] is [RuntimeStatus::Created] or
    /// [RuntimeStatus::Running] on Linux, OPTIONAL on other platforms
    pid: u64,

    /// The absolute path to the container's bundle directory. This is provided
    /// so that consumers can find the container's configuration and root
    /// filesystem on the host.
    bundle: PathBuf,

    /// contains the list of annotations associated with the container. If no
    /// annotations were provided then this property MAY either be absent or an
    /// empty map.
    annotations: Option<HashMap<String, String>>,
}

/// An OCI runtime will have to implement this trait.
pub trait Runtime {
    type Err;

    /// Query State
    ///
    /// This operation MUST generate an error if it is not provided the ID of a
    /// container. Attempting to query a container that does not exist MUST
    /// generate an error. This operation MUST return the state of a container
    /// as specified in the State section.
    fn state(&self) -> Result<State, Self::Err>;

    /// Create
    ///
    /// This operation MUST generate an error if it is not provided a path to
    /// the bundle and the container ID to associate with the container. If the
    /// ID provided is not unique across all containers within the scope of the
    /// runtime, or is not valid in any other way, the implementation MUST
    /// generate an error and a new container MUST NOT be created. This
    /// operation MUST create a new container.
    ///
    /// All of the properties configured in `config.json` except for `process`
    /// MUST be applied. `process.args` MUST NOT be applied until triggered by
    /// the start operation. The remaining `process` properties MAY be applied
    /// by this operation. If the runtime cannot apply a property as specified
    /// in the configuration, it MUST generate an error and a new container
    /// MUST NOT be created.
    ///
    /// The runtime MAY validate `config.json` against this spec, either
    /// generically or with respect to the local system capabilities, before
    /// creating the container (step 2). Runtime callers who are interested
    /// in pre-create validation can run bundle-validation tools before invoking
    /// the create operation.
    ///
    /// Any changes made to the `config.json` file after this operation will not
    /// have an effect on the container.
    fn create(&mut self, path_to_bundle: Path) -> Result<(), Self::Err>;

    /// Start
    ///
    /// This operation MUST generate an error if it is not provided th
    ///container ID. Attempting to [start] a container that is not
    /// [RuntimeState::Created] MUST have no effect on the container and MUST
    /// generate an error. This operation MUST run the user-specified program as
    /// specified by `process`. This operation MUST generate an error if
    /// `process` was not set.
    fn start(&mut self) -> Result<(), Self::Err>;

    /// Kill
    ///
    /// This operation MUST generate an error if it is not provided the
    /// container ID. Attempting to send a signal to a container that is neither
    /// [RuntimeState::Created] nor [RuntimeState::Running] MUST have no effect
    /// on the container and MUST generate an error. This operation MUST send
    /// the specified signal to the container process.
    // FIXME: use better signal type here
    fn kill(&mut self, signal: u16) -> Result<(), Self::Err>;

    /// Delete
    ///
    /// This operation MUST generate an error if it is not provided the
    /// container ID. Attempting to [delete] a container that is not
    /// [RuntimeState::Stopped] MUST have no effect on the container and MUST
    /// generate an error. Deleting a container MUST delete the resources that
    /// were created during the create step. Note that resources associated with
    /// the container, but not created by this container, MUST NOT be deleted.
    /// Once a container is deleted its ID MAY be used by a subsequent container.
    fn delete(self) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_runtime_state() {
        let test_data = include_str!("test/state.test.json");

        let state: State = serde_json::from_str(test_data).expect("Could not deserialize state");

        assert_eq!(state.id, "oci-container1");
        assert_eq!(state.pid, 4422);
        assert_eq!(state.bundle, PathBuf::from("/containers/redis"));
    }
}
