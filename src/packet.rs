//! Contains tools related to packets.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    name: String,
    password: String,
    #[serde(default = "default_false")]
    password_hashed: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Authentication {
    admins: Vec<User>,
    participants: Vec<User>,
}

/// Represents a Packet containing questions and
/// configurations
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Packet {
    title: Option<String>,
    preamble: Option<String>,
    default_language: Option<String>,
    languages: Option<Vec<String>>,
    problems: Vec<Problem>,
    authentication: Authentication,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Problem {
    default_language: Option<String>,
    languages: Option<String>,
    title: Option<String>,
    tests: Vec<Test>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Test {
    input: String,
    output: String,
    #[serde(default = "default_false")]
    visible: bool,
}

fn default_false() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packets_parse_correctly() {
        // parse example one
        let pkt: Packet = toml::from_str(include_str!("../examples/one.toml")).unwrap();
    }
}
