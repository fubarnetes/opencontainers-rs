use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub config: RuntimeConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// MUST be in SemVer v2.0.0 format and specifies the version of the Open
    /// Container Initiative Runtime Specification with which the bundle
    /// complies. The Open Container Initiative Runtime Specification follows
    /// semantic versioning and retains forward and backward compatibility
    /// within major versions. For example, if a configuration is compliant with
    /// version 1.1 of this specification, it is compatible with all runtimes
    /// that support any 1.1 or later release of this specification, but is not
    /// compatible with a runtime that supports 1.0 and not 1.1.
    // FIXME: This should probably be a semver::Version
    #[serde(rename = "ociVersion")]
    pub oci_version: String,

    /// specifies the container's root filesystem. On Windows, for Windows
    /// Server Containers, this field is REQUIRED. For Hyper-V Containers,
    /// his field MUST NOT be set.
    ///
    /// On all other platforms, this field is REQUIRED.
    pub root: Option<Root>,

    /// specifies additional mounts beyond `root`. The runtime MUST mount
    /// entries in the listed order. For Linux, the parameters are as documented
    /// in mount(2) system call man page. For Solaris, the mount entry
    /// corresponds to the 'fs' resource in the zonecfg(1M) man page.
    pub mounts: Option<Vec<Mount>>,

    /// specifies the container process. This property is REQUIRED when
    /// [Runtime::start] is called.
    pub process: Option<Process>,

    /// specifies the container's hostname as seen by processes running inside
    /// the container. On Linux, for example, this will change the hostname in
    /// the container UTS namespace. Depending on your namespace configuration,
    /// the container UTS namespace may be the runtime UTS namespace.
    pub hostname: Option<String>,

    /// FIXME: Add Platform-specific configuration
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    /// Specifies the path to the root filesystem for the container.
    ///
    /// * On Windows, `path` MUST be a volume GUID path.
    /// * On POSIX platforms, `path` is either an absolute path or a relative
    ///   path to the bundle. For example, with a bundle at `/to/bundle` and a
    ///   root filesystem at `/to/bundle/rootfs`, the path value can be either
    ///   `/to/bundle/rootfs` or `rootfs`. The value SHOULD be the conventional
    ///   rootfs.
    pub path: PathBuf,

    /// If true then the root filesystem MUST be read-only inside the container,
    /// defaults to false.
    ///
    /// On Windows, this field MUST be omitted or false.
    pub readonly: Option<bool>,
}

impl Root {
    pub fn readonly(&self) -> bool {
        match self.readonly {
            None => false,
            Some(v) => v,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mount {
    /// Destination of mount point: path inside container.
    ///
    /// This value MUST be an absolute path.
    ///
    /// * Windows: one mount destination MUST NOT be nested within another mount
    ///   (e.g., c:\foo and c:\foo\bar).
    /// * Solaris: corresponds to "dir" of the fs resource in zonecfg(1M)
    pub destination: PathBuf,

    /// A device name, but can also be a file or directory name for bind mounts
    /// or a dummy.
    ///
    /// Path values for bind mounts are either absolute or relative to the
    /// bundle. A mount is a bind mount if it has either `bind` or `rbind` in
    /// the options.
    ///
    /// * Windows: a local directory on the filesystem of the container host.
    ///   UNC paths and mapped drives are not supported.
    /// * Solaris: corresponds to "special" of the fs resource in zonecfg(1M).
    pub source: Option<PathBuf>,

    /// Mount options of the filesystem to be used.
    ///
    /// * Linux: supported options are listed in the mount(8) man page. Note
    ///   both filesystem-independent and filesystem-specific options are listed.
    /// * Solaris: corresponds to "options" of the fs resource in zonecfg(1M).
    /// * Windows: runtimes MUST support ro, mounting the filesystem read-only
    ///   when ro is given.
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    /// specifies whether a terminal is attached to the process, defaults to
    /// false. As an example, if set to true on Linux a pseudoterminal pair is
    /// allocated for the process and the pseudoterminal slave is duplicated on
    /// the process's standard streams.
    pub terminal: Option<bool>,

    /// specifies the console size in characters of the terminal.
    ///
    /// Runtimes MUST ignore [console_size] if [terminal] is `false`or unset.
    #[serde(rename = "consoleSize")]
    pub console_size: Option<ConsoleSize>,

    /// the working directory that will be set for the executable.
    ///
    /// This value MUST be an absolute path.
    pub cwd: PathBuf,

    /// array of strings with the same semantics as IEEE Std 1003.1-2008's
    /// `environ`.
    pub env: Option<Vec<String>>,

    /// array of strings with similar semantics to IEEE Std 1003.1-2008 execvp's
    /// `argv`. This specification extends the IEEE standard in that at least
    /// one entry is REQUIRED (non-Windows), and that entry is used with the
    /// same semantics as `execvp`'s file. This field is OPTIONAL on Windows,
    /// and commandLine is REQUIRED if this field is omitted.
    pub args: Option<Vec<String>>,

    /// specifies the full command line to be executed on Windows.
    ///
    /// This is the preferred means of supplying the command line on Windows. If
    /// omitted, the runtime will fall back to escaping and concatenating fields
    /// from args before making the system call into Windows.
    pub command_line: Option<String>,

    /// The user for the process is a platform-specific structure that allows
    /// specific control over which user the process runs as.
    user: User,

    #[serde(flatten)]
    pub posix: PosixProcessExt,

    #[serde(flatten)]
    pub linux: LinuxProcessExt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PosixProcessExt {
    /// Allows setting resource limits for the process.AsMut
    /// If `rlimits` contains duplicated entries with same type, the runtime
    /// MUST generate an error.
    pub rlimits: Option<Vec<RLimit>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinuxProcessExt {
    /// specifies the name of the AppArmor profile for the process.
    ///
    /// For more information about AppArmor, see [AppArmor documentation].
    ///
    /// [AppArmor documentation]: https://wiki.ubuntu.com/AppArmor
    #[serde(rename = "apparmorProfile")]
    pub apparmor_profile: Option<String>,

    /// An object containing arrays that specifies the sets of capabilities for
    /// the process. Valid values are defined in the capabilities(7) man page,
    /// such as `CAP_CHOWN`. Any value which cannot be mapped to a relevant
    /// kernel interface MUST cause an error.
    pub capabilities: Option<Capabilities>,

    /// prevents the process from gaining additional privileges. As an example,
    /// the `no_new_privs` article in the kernel documentation has information
    /// on how this is achieved using a `prctl` system call on Linux.
    #[serde(rename = "noNewPrivileges")]
    pub no_new_privileges: Option<bool>,

    /// Adjusts the oom-killer score in `[pid]/oom_score_adj` for the process's
    /// `[pid]` in a proc pseudo-filesystem. If `oomScoreAdj` is set, the
    /// runtime MUST set `oom_score_adj` to the given value.
    /// If `oomScoreAdj` is not set, the runtime MUST NOT change the value of
    /// `oom_score_adj`.
    ///
    /// This is a per-process setting, where as `disableOOMKiller` is scoped for
    /// a memory cgroup. For more information on how these two settings work
    /// together, see the memory cgroup documentation section 10. OOM Contol.
    #[serde(rename = "oomScoreAdj")]
    pub oom_score_adj: Option<i64>,

    /// specifies the SELinux label for the process. For more information about
    /// SELinux, see [SELinux documentation].
    ///
    /// [Selinux documentation]: http://selinuxproject.org/page/Main_Page
    #[serde(rename = "selinuxLabel")]
    pub selinux_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RLimit {
    /// The platform resource being limited.
    ///
    /// * Linux: valid values are defined in the getrlimit(2) man page, such as
    ///   `RLIMIT_MSGQUEUE`.
    /// * Solaris: valid values are defined in the getrlimit(3) man page, such
    ///   as `RLIMIT_CORE`.
    ///
    /// The runtime MUST generate an error for any values which cannot be mapped
    /// to a relevant kernel interface. For each entry in `rlimits`, a
    /// `getrlimit(3)` on `type` MUST succeed. For the following properties,
    /// `rlim` refers to the status returned by the `getrlimit(3)` call.
    pub r#type: String,

    /// The value of the limit enforced for the corresponding resource.
    /// `rlim.rlim_cur` MUST match the configured value.
    pub soft: u64,

    /// The ceiling for the soft limit that could be set by an unprivileged
    /// process. `rlim.rlim_max` MUST match the configured value. Only a
    /// privileged process (e.g. one with the `CAP_SYS_RESOURCE` capability) can
    /// raise a hard limit.
    pub hard: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Capabilities {
    /// Effective capabilities that are kept for the process.
    pub effective: Option<Vec<String>>,

    /// Bounding capabilities that are kept for the process.
    pub bounding: Option<Vec<String>>,

    /// Inheritable capabilities that are kept for the process.
    pub inheritable: Option<Vec<String>>,

    /// Permitted capabilities that are kept for the process.
    pub permitted: Option<Vec<String>>,

    /// Ambient capabilities that are kept for the process.
    pub ambient: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(flatten)]
    pub posix: Option<PosixUser>,

    #[serde(flatten)]
    pub windows: Option<WindowsUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PosixUser {
    /// specifies the user ID in the container namespace.
    pub uid: u64,

    /// specifies the group ID in the container namespace.
    pub gid: u64,

    /// specifies additional group IDs in the container namespace to be added
    /// to the process.
    #[serde(rename = "additionalGids")]
    pub additional_gids: Option<Vec<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsUser {
    /// specifies the user name for the process.
    pub username: Option<String>,
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
    type Err = !;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_runtime_config() {
        let test_data = include_str!("test/config.test.json");

        let config: RuntimeConfig =
            serde_json::from_str(test_data).expect("Could not deserialize config");

        assert!(config.process.is_some());

        if let Some(ref process) = config.process {
            assert_eq!(process.terminal, Some(true));
            assert_eq!(process.cwd, PathBuf::from("/"));
        }

        assert_eq!(config.hostname.unwrap(), "slartibartfast");

        assert!(config.root.is_some());

        if let Some(ref root) = config.root {
            assert_eq!(root.path, PathBuf::from("rootfs"));
            assert_eq!(root.readonly(), true);
        }
    }

    #[test]
    fn test_runtime_state() {
        let test_data = include_str!("test/state.test.json");

        let state: State = serde_json::from_str(test_data).expect("Could not deserialize state");

        assert_eq!(state.id, "oci-container1");
        assert_eq!(state.pid, 4422);
        assert_eq!(state.bundle, PathBuf::from("/containers/redis"));
    }
}
