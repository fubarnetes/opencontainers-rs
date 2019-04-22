//! Rust enums for Go (Golang) values of GOOS and GOARCH

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Fail)]
pub enum GoError {
    #[fail(display = "Invalid GOOS string: {}", _0)]
    InvalidGoOs(String),

    #[fail(display = "Invalid GOARCH string: {}", _0)]
    InvalidGoArch(String),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum GoOs {
    Android,
    Darwin,
    Dragonfly,
    FreeBSD,
    Linux,
    NaCl,
    NetBSD,
    OpenBSD,
    Plan9,
    Solaris,
    Windows,
    ZOS,
}

impl std::str::FromStr for GoOs {
    type Err = GoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "android" => Ok(GoOs::Android),
            "darwin" => Ok(GoOs::Darwin),
            "dragonfly" => Ok(GoOs::Dragonfly),
            "freebsd" => Ok(GoOs::FreeBSD),
            "linux" => Ok(GoOs::Linux),
            "nacl" => Ok(GoOs::NaCl),
            "netbsd" => Ok(GoOs::NetBSD),
            "openbsd" => Ok(GoOs::OpenBSD),
            "plan9" => Ok(GoOs::Plan9),
            "solaris" => Ok(GoOs::Solaris),
            "windows" => Ok(GoOs::Windows),
            "zos" => Ok(GoOs::ZOS),
            other => Err(GoError::InvalidGoOs(other.into())),
        }
    }
}

impl std::fmt::Display for GoOs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GoOs::Android => "android",
                GoOs::Darwin => "darwin",
                GoOs::Dragonfly => "dragonfly",
                GoOs::FreeBSD => "freebsd",
                GoOs::Linux => "linux",
                GoOs::NaCl => "nacl",
                GoOs::NetBSD => "netbsd",
                GoOs::OpenBSD => "openbsd",
                GoOs::Plan9 => "plan9",
                GoOs::Solaris => "solaris",
                GoOs::Windows => "windows",
                GoOs::ZOS => "zos",
            }
        )
    }
}

impl<'de> Deserialize<'de> for GoOs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for GoOs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum GoArch {
    I386,
    AMD64,
    AMD64p32,
    ARM,
    ARMbe,
    ARM64,
    ARM64be,
    PPC64,
    PPC64le,
    MIPS,
    MIPSle,
    MIPS64,
    MIPS64le,
    MIPS64p32,
    MIPS64p32le,
    PPC,
    S390,
    S390x,
    SPARC,
    SPARC64,
}

impl std::str::FromStr for GoArch {
    type Err = GoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "386" => Ok(GoArch::I386),
            "amd64" => Ok(GoArch::AMD64),
            "amd64p32" => Ok(GoArch::AMD64p32),
            "arm" => Ok(GoArch::ARM),
            "armbe" => Ok(GoArch::ARMbe),
            "arm64" => Ok(GoArch::ARM64),
            "arm64be" => Ok(GoArch::ARM64be),
            "ppc64" => Ok(GoArch::PPC64),
            "ppc64le" => Ok(GoArch::PPC64le),
            "mips" => Ok(GoArch::MIPS),
            "mipsle" => Ok(GoArch::MIPSle),
            "mips64" => Ok(GoArch::MIPS64),
            "mips64le" => Ok(GoArch::MIPS64le),
            "mips64p32" => Ok(GoArch::MIPS64p32),
            "mips64p32le" => Ok(GoArch::MIPS64p32le),
            "ppc" => Ok(GoArch::PPC),
            "s390" => Ok(GoArch::S390),
            "s390x" => Ok(GoArch::S390x),
            "sparc" => Ok(GoArch::SPARC),
            "sparc64" => Ok(GoArch::SPARC64),
            other => Err(GoError::InvalidGoArch(other.into())),
        }
    }
}

impl std::fmt::Display for GoArch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GoArch::I386 => "386",
                GoArch::AMD64 => "amd64",
                GoArch::AMD64p32 => "amd64p32",
                GoArch::ARM => "arm",
                GoArch::ARMbe => "armbe",
                GoArch::ARM64 => "arm64",
                GoArch::ARM64be => "arm64be",
                GoArch::PPC64 => "ppc64",
                GoArch::PPC64le => "ppc64le",
                GoArch::MIPS => "mips",
                GoArch::MIPSle => "mipsle",
                GoArch::MIPS64 => "mips64",
                GoArch::MIPS64le => "mips64le",
                GoArch::MIPS64p32 => "mips64p32",
                GoArch::MIPS64p32le => "mips64p32le",
                GoArch::PPC => "ppc",
                GoArch::S390 => "s390",
                GoArch::S390x => "s390x",
                GoArch::SPARC => "sparc",
                GoArch::SPARC64 => "sparc64",
            }
        )
    }
}

impl<'de> Deserialize<'de> for GoArch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for GoArch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}
