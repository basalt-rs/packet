use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use miette::NamedSource;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::ConfigReadError;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct RawOrImport<T>(T);

impl<'de, T> Deserialize<'de> for RawOrImport<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO:
        // These gems are stolen from the generated output of `#[derive(Deserialize)]`
        // This is obviously not ideal (literally using `__private`) so we should look at how to
        // properly do this in the future.
        let content = serde::__private::de::Content::deserialize(deserializer)?;
        let de = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);

        if let Ok(import) = Import::deserialize(de) {
            // TODO: Figure out how to make the path relative to the toml file rather than the
            // runtime
            // TODO: This sync code makes me want to die
            let content =
                std::fs::read_to_string(&import.import).map_err(serde::de::Error::custom)?;

            let x: T = toml_edit::de::from_str(&content)
                .map_err(|e| {
                    ConfigReadError::malformed(
                        NamedSource::new(import.import.display().to_string(), content),
                        e,
                    )
                })
                .map_err(serde::de::Error::custom)?;
            return Ok(Self(x));
        }
        Ok(Self(T::deserialize(de)?))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
struct Import {
    import: PathBuf,
}

impl<T> Deref for RawOrImport<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RawOrImport<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for RawOrImport<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
