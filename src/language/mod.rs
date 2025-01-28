mod language_set;
pub use language_set::LanguageSet;

use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

static LANG_NAMES: phf::Map<&'static str, BuiltInLanguage> = phf_map! {
    "python3" => BuiltInLanguage::Python3,
    "java" => BuiltInLanguage::Java,
    "javascript" => BuiltInLanguage::JavaScript,
    "rust" => BuiltInLanguage::Rust,
};

struct CommandCombo {
    build: &'static str,
    run: &'static str,
}

// version : (build command, run command)
static JAVA_VERSIONS: phf::Map<&'static str, CommandCombo> = phf_map! {
    "8" => CommandCombo { build: "/lib/jvm/java-8-openjdk/bin/javac Solution.java", run: "/lib/jvm/java-8-openjdk/bin/java Solution" },
    "11" => CommandCombo { build: "/lib/jvm/java-11-openjdk/bin/javac Solution.java", run: "/lib/jvm/java-11-openjdk/bin/java Solution" },
    "23" => CommandCombo { build: "/lib/jvm/java-23-openjdk/bin/javac Solution.java", run: "/lib/jvm/java-23-openjdk/bin/java Solution" },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum BuiltInLanguage {
    Python3,
    Java,
    JavaScript,
    Rust,
}

impl BuiltInLanguage {
    pub const BUILTINS: [&'static str; 4] = [
        Self::Python3.as_str(),
        Self::Java.as_str(),
        Self::JavaScript.as_str(),
        Self::Rust.as_str(),
    ];

    pub fn has_version(self, version: Version) -> bool {
        match (self, version) {
            (Self::Python3, Version::Latest) => true,
            (Self::Python3, _) => false,
            (Self::Java, Version::Latest) => true,
            (Self::Java, Version::Specific(v)) => JAVA_VERSIONS.contains_key(&v),
            (Self::JavaScript, Version::Latest) => true,
            (Self::JavaScript, _) => false,
            (Self::Rust, Version::Latest) => true,
            (Self::Rust, _) => false,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Python3 => "python3",
            Self::Java => "java",
            Self::JavaScript => "javascript",
            Self::Rust => "rust",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Python3 => "Python3",
            Self::Java => "Java",
            Self::JavaScript => "JavaScript",
            Self::Rust => "Rust",
        }
    }

    pub const fn source_file(self) -> &'static str {
        match self {
            BuiltInLanguage::Python3 => "solution.py",
            BuiltInLanguage::Java => "Solution.java",
            BuiltInLanguage::JavaScript => "solution.js",
            BuiltInLanguage::Rust => "solution.rs",
        }
    }

    pub fn build_command(self, version: &Version) -> Option<&str> {
        match self {
            Self::Python3 => None,
            Self::Java => match version {
                Version::Latest => Some(JAVA_VERSIONS["21"].build),
                Version::Specific(v) => Some(JAVA_VERSIONS[v].build),
            },
            Self::JavaScript => None,
            Self::Rust => Some("rustc -o solution solution.rs"),
        }
    }

    pub fn run_command(self, version: &Version) -> &str {
        match self {
            Self::Python3 => "python3 solution.py",
            Self::Java => match version {
                Version::Latest => JAVA_VERSIONS["21"].run,
                Version::Specific(v) => JAVA_VERSIONS[v].run,
            },
            Self::JavaScript => "node solution.js",
            Self::Rust => "./solution",
        }
    }
}

impl From<&str> for BuiltInLanguage {
    fn from(value: &str) -> Self {
        LANG_NAMES[value]
    }
}

impl FromStr for BuiltInLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LANG_NAMES.get(s).ok_or(()).copied()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Version {
    Latest,
    Specific(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Language {
    BuiltIn {
        language: BuiltInLanguage,
        version: Version,
    },
    Custom {
        raw_name: String,
        name: String,
        build: Option<String>,
        run: String,
        source_file: String,
    },
}

impl Language {
    pub fn name(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.name(),
            Language::Custom { name, .. } => name,
        }
    }

    pub fn source_file(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.source_file(),
            Language::Custom { source_file, .. } => source_file,
        }
    }

    pub fn build_command(&self) -> Option<&str> {
        match self {
            Language::BuiltIn { language, version } => language.build_command(version),
            Language::Custom { build, .. } => build.as_deref(),
        }
    }

    pub fn run_command(&self) -> &str {
        match self {
            Language::BuiltIn { language, version } => language.run_command(version),
            Language::Custom { run, .. } => run,
        }
    }
}
