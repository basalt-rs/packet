//! Contains tools related to packets.

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packets_parse_correctly() {
        // parse example one
        let _: Packet = toml::from_str(include_str!("../examples/one.toml")).unwrap();
    }
}
