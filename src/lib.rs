use std::{collections::BTreeMap, fs, io::Read, path::PathBuf};

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use packet::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub mod packet;

#[cfg(test)]
mod tests;

pub(crate) fn default_false() -> bool {
    false
}

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(untagged)] // TODO: This is mildly fucking up the error messages
pub enum RawOrImport<T> {
    /// The type has been directly stated in the current file
    Raw(T),
    /// The contents have been placed into another file
    Import { import: PathBuf },
}

impl<T> RawOrImport<T>
where
    T: DeserializeOwned,
{
    /// Get the contents of this item if it's [`RawOrImport::Raw`], otherwise get the contents from
    /// file
    pub fn get(self) -> Result<T, ConfigReadError> {
        match self {
            RawOrImport::Raw(t) => Ok(t),
            RawOrImport::Import { import } => {
                let content = fs::read_to_string(&import)?;
                toml_edit::de::from_str(&content).map_err(|e| {
                    ConfigReadError::malformed(
                        NamedSource::new(import.display().to_string(), content)
                            .with_language("TOML"),
                        e,
                    )
                })
            }
        }
    }

    /// Get the contents of this item if it's [`RawOrImport::Raw`], otherwise get the contents from
    /// file
    #[cfg(feature = "tokio")]
    pub async fn get_async(self) -> Result<T, ConfigReadError> {
        match self {
            RawOrImport::Raw(t) => Ok(t),
            RawOrImport::Import { import } => {
                let content = tokio::fs::read_to_string(&import).await?;
                toml_edit::de::from_str(&content).map_err(|e| {
                    ConfigReadError::malformed(
                        NamedSource::new(import.display().to_string(), content),
                        e,
                    )
                })
            }
        }
    }
}

impl<T> From<T> for RawOrImport<T> {
    fn from(value: T) -> Self {
        Self::Raw(value)
    }
}

/// Authentication details for a specific user (competitor or admin)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub name: String,
    pub password: String,
}

/// Set of users that are either hosts or competitors
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct Accounts {
    /// Administrators in charge of managing the competition
    pub admins: Vec<User>,
    /// Competitors participating in the competition
    pub competitors: Vec<User>,
}

/// Configuration for setting up the docker container and starting the server
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    /// Specifies what commands are to be run when building the container to ensure dependencies
    /// are installed.
    pub install: Option<RawOrImport<String>>,
    /// Specifies commands to run before running basalt-server so that dependencies are enabled
    /// properly.
    pub init: Option<RawOrImport<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Language {
    /// Use the recommended version of this language
    #[serde(alias = "*")]
    Enabled,
    /// A language that we do not have a configuration for
    #[serde(untagged)]
    Custom {
        // TODO: Custom command deserialiser
        name: Option<String>,
        build: Option<String>,
        run: String,
    },
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum ConfigReadError {
    /// The Config file was unable to be read due to an IO error
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
    /// The data being deserialised was formatted incorrectly
    #[error("{}", .0.to_string())] // needed to use the miette error instead of thiserror
    #[diagnostic(transparent)]
    MalformedData(miette::Error),
}

impl ConfigReadError {
    fn malformed<S>(source: S, value: toml_edit::de::Error) -> Self
    where
        S: SourceCode + 'static,
    {
        let labels = if let Some(span) = value.span() {
            vec![LabeledSpan::new_with_span(Some("here".into()), span)]
        } else {
            Vec::new()
        };
        Self::MalformedData(
            miette::miette! {
                labels = labels,
                "{}", value.message()
            }
            .with_source_code(source),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration for setting up the docker container and starting the server
    pub setup: Option<RawOrImport<Setup>>,
    /// Port on which the server will be hosted
    pub port: u16,
    /// List of languages available for the server
    pub languages: BTreeMap<String, Language>,
    /// Accounts that will be granted access to the server
    pub accounts: Accounts,
    /// The packet for this competition
    pub packet: Packet,
}

impl Config {
    /// Read config from a string
    ///
    /// - `file_name` provided for better miette errors
    pub fn from_str(
        content: impl AsRef<str>,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError> {
        toml_edit::de::from_str(content.as_ref()).map_err(|e| {
            if let Some(file_name) = file_name {
                ConfigReadError::malformed(
                    NamedSource::new(file_name, content.as_ref().to_string()).with_language("TOML"),
                    e,
                )
            } else {
                ConfigReadError::malformed(content.as_ref().to_string(), e)
            }
        })
    }

    /// Read config from a file
    ///
    /// - `file_name` provided for better miette errors
    pub fn read<R>(
        reader: &mut R,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError>
    where
        R: Read,
    {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Self::from_str(&buf, file_name)
    }

    /// Read config from a file asynchronously
    ///
    /// - `file_name` provided for better miette errors
    #[cfg(feature = "tokio")]
    pub async fn read_async<R>(
        reader: &mut R,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        use tokio::io::AsyncReadExt;
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await?;
        Self::from_str(&buf, file_name)
    }
}
