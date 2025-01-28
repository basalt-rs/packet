use std::{fs, path::PathBuf};

use miette::NamedSource;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::ConfigReadError;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
pub enum RawOrImport<T> {
    /// The type has been directly stated in the current file
    Raw(T),
    /// The contents have been placed into another file
    Import { import: PathBuf },
}

impl<'de, T> Deserialize<'de> for RawOrImport<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO:
        // These gems are stolen from the generated output of `#[derive(Deserialize)]`
        // This is probably not the ideal solution, but it works, so we may want to change this in
        // the future.
        let content = serde::__private::de::Content::deserialize(deserializer)?;
        let de = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);

        if let Ok(import) = Import::deserialize(de) {
            return Ok(RawOrImport::Import {
                import: import.import,
            });
        }
        Ok(RawOrImport::Raw(T::deserialize(de)?))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
struct Import {
    import: PathBuf,
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
