use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

use miette::NamedSource;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::ConfigReadError;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[non_exhaustive]
pub struct Deser;
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[non_exhaustive]
pub struct Raw;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct RawOrImport<T, Mode = Deser>(T, PhantomData<Mode>)
where
    Mode: Sized;

impl<'de, T> Deserialize<'de> for RawOrImport<T, Deser>
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
            return Ok(Self(x, PhantomData));
        }
        Ok(Self(T::deserialize(de)?, PhantomData))
    }
}

impl<'de, S> Deserialize<'de> for RawOrImport<S, Raw>
where
    S: FromStr + Deserialize<'de>,
    S::Err: std::fmt::Display,
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

            return Ok(Self(
                content.parse().map_err(serde::de::Error::custom)?,
                PhantomData,
            ));
        }
        Ok(Self(S::deserialize(de)?, PhantomData))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
struct Import {
    import: PathBuf,
}

impl<T, Mode> Deref for RawOrImport<T, Mode> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, Mode> DerefMut for RawOrImport<T, Mode> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, Mode> From<T> for RawOrImport<T, Mode> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}
