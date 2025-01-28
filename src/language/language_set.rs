use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::language::Version;

use super::Language;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LanguageSet {
    inner: HashSet<Language>,
}

impl LanguageSet {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashSet::with_capacity(capacity),
        }
    }
}

impl Deref for LanguageSet {
    type Target = HashSet<Language>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for LanguageSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

struct LanguageMapVisitor;

impl<'de> Visitor<'de> for LanguageMapVisitor {
    type Value = LanguageSet;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a map of languages")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = LanguageSet::with_capacity(access.size_hint().unwrap_or(0));

        // TODO: Spans or something for better error messages
        while let Some((key, value)) = access.next_entry::<Cow<'_, str>, TomlLanguage>()? {
            let val = match value {
                TomlLanguage::Latest => Language::BuiltIn {
                    language: key.parse().map_err(|()| {
                        serde::de::Error::custom(format!("Unknown built-in language: '{}'", key))
                    })?,
                    version: Version::Latest,
                },
                TomlLanguage::Version(v) => Language::BuiltIn {
                    language: key.parse().map_err(|()| {
                        serde::de::Error::custom(format!("Unknown built-in language: '{}'", key))
                    })?,
                    version: Version::Specific(v.into()), // TODO: Enforce the language version here
                },
                TomlLanguage::Custom {
                    name,
                    build,
                    run,
                    source_file,
                } => Language::Custom {
                    name: name.unwrap_or_else(|| key.clone()).into_owned(),
                    raw_name: key.into_owned(),
                    build: build.map(Cow::into_owned),
                    run: run.into_owned(),
                    source_file: source_file.into_owned(),
                },
            };

            map.insert(val);
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for LanguageSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LanguageMapVisitor)
    }
}

impl Serialize for LanguageSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for lang in &self.inner {
            match lang {
                Language::BuiltIn {
                    language: name,
                    version: value,
                } => {
                    map.serialize_entry(name.as_str(), &TomlLanguage::from(value))?;
                }
                Language::Custom {
                    raw_name,
                    name,
                    build,
                    run,
                    source_file,
                } => {
                    map.serialize_entry(
                        raw_name,
                        &TomlLanguage::Custom {
                            name: Some(name.into()),
                            build: build.as_ref().map(Into::into),
                            run: run.into(),
                            source_file: source_file.into(),
                        },
                    )?;
                }
            }
        }
        map.end()
    }
}

/// Language as represented in the toml file
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
enum TomlLanguage<'a> {
    #[serde(alias = "*", alias = "enabled")]
    Latest,
    #[serde(untagged)]
    Version(Cow<'a, str>),
    #[serde(untagged)]
    Custom {
        // TODO: Custom command deserialiser
        name: Option<Cow<'a, str>>,
        build: Option<Cow<'a, str>>,
        run: Cow<'a, str>,
        source_file: Cow<'a, str>,
    },
}

impl<'a> From<&'a Version> for TomlLanguage<'a> {
    fn from(value: &'a Version) -> Self {
        match value {
            Version::Latest => TomlLanguage::Latest,
            Version::Specific(v) => TomlLanguage::Version(v.into()),
        }
    }
}
