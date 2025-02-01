use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::RawOrImport;

/// Structure represnting data for a problem
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Problem {
    /// The languages that may be used to solve this question
    ///
    /// Must be a subset of the languages listed in the Config
    pub languages: Option<HashSet<String>>,
    /// The title for this specific problem
    pub title: String,
    /// The description of this problem (supports markdown)
    pub description: Option<RawOrImport<String>>,
    /// The tests that will be used on this problem
    pub tests: Vec<Test>,
}

/// A specific test that will be used to validate that user's code.
///
/// The input and expected output for visible tests will be shown to the user
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(deny_unknown_fields)]
pub struct Test {
    /// The input that will be provided via STDIN to the test
    pub input: String,
    /// The expected output from STDOUT
    pub output: String,
    /// Whether the test runner should trim the output of the test before comparing with the
    /// expected output
    ///
    /// If this is true, the output of `hello world    ` matches the expected output of ` hello
    /// world`
    ///
    /// [Default: true]
    #[serde(default = "crate::default_true")]
    pub trim_output: bool,
    /// Whether this test should be shown to the competitor or just used for validation
    ///
    /// The first visible test will be shown as an example for the user
    #[serde(default = "crate::default_false")]
    pub visible: bool,
}

/// A packet which contains configuration for problems and their tests
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    /// Title of the packet
    pub title: String,
    /// Information about the packet that will be included at the top of the file
    pub preamble: Option<RawOrImport<String>>,
    /// The list of problems for this
    pub problems: Vec<RawOrImport<Problem>>,
}
