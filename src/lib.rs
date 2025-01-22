//! Contains tools related to packets.

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub name: String,
    pub password: String,
    #[serde(default = "default_false")]
    pub password_hashed: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Authentication {
    pub admins: Vec<User>,
    pub participants: Vec<User>,
}

/// Represents a Packet containing questions and
/// configurations
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Packet {
    pub title: String,
    pub preamble: Option<String>,
    pub default_language: Option<String>,
    pub languages: Option<Vec<String>>,
    pub problems: Vec<Problem>,
    pub authentication: Authentication,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Problem {
    pub default_language: Option<String>,
    pub languages: Option<String>,
    pub title: Option<String>,
    pub tests: Vec<Test>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Test {
    pub input: String,
    pub output: String,
    #[serde(default = "default_false")]
    pub visible: bool,
}

fn default_false() -> bool {
    false
}

#[derive(Debug, thiserror::Error)]
pub enum PacketReadError {
    #[error("Failed to read packet file: {0}")]
    FailedToReadPacketFile(std::io::Error),
    #[error("Packet is malformed: {0}")]
    MalformedPacketError(String),
}

impl TryFrom<PathBuf> for Packet {
    type Error = PacketReadError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        // read content from file
        let content_bytes =
            fs::read(value).map_err(|e| PacketReadError::FailedToReadPacketFile(e))?;
        let content = std::str::from_utf8(&content_bytes)
            .map_err(|e| PacketReadError::MalformedPacketError(e.to_string()))?;
        let packet: Packet = toml::from_str(content)
            .map_err(|e| PacketReadError::MalformedPacketError(e.to_string()))?;
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, path::PathBuf};
    use tempfile::NamedTempFile;
    const EXAMPLE_ONE_CONTENT: &str = include_str!("../examples/one.toml");
    #[test]
    fn packets_parse_correctly() {
        // parse example one
        let _: Packet = toml::from_str(EXAMPLE_ONE_CONTENT).unwrap();
    }
    #[test]
    fn packet_files_parse_correctly() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(EXAMPLE_ONE_CONTENT.as_bytes()).unwrap();
        let _: Packet = Packet::try_from(PathBuf::from(file.path())).unwrap();
    }
}
