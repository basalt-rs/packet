//! Contains tools related to packets.

use serde::{Deserialize, Serialize};

/// Represents a Packet containing questions and
/// configurations
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Packet {
    title: Option<String>,
    preamble: Option<String>,
    default_language: Option<String>,
    languages: Option<Vec<String>>,
    problems: Vec<Problem>,
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
    #[serde(default = "default_visibility")]
    visible: bool,
}

fn default_visibility() -> bool {
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
