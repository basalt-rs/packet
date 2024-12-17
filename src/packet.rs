//! Contains tools related to packets.

use serde::{Deserialize, Serialize};

/// Represents a Packet containing questions and
/// configurations
#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    default_language: Option<String>,
    languages: Option<String>,
    problems: Vec<Problem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    default_language: Option<String>,
    languages: Option<String>,
    title: Option<String>,
    tests: Vec<Test>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Test {
    input: String,
    output: String,
    #[serde(default = "default_visibility")]
    visible: bool,
}

fn default_visibility() -> bool {
    false
}
