use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{
    de::{Deserialize, Deserializer, Visitor},
    ser::Serialize,
};

#[derive(Debug, Clone)]
pub(crate) struct RelativePathBuf(PathBuf);

impl RelativePathBuf {
    pub(crate) fn as_path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn into_pathbuf(self) -> PathBuf {
        self.0
    }
}

impl TryFrom<&str> for RelativePathBuf {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        let path = PathBuf::from(value);

        path.is_relative()
            .then_some(Self(path))
            .context("cannot create RelativePathBuf: Path is not relative")
    }
}

impl<'de> Deserialize<'de> for RelativePathBuf {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierVisitor;

        impl<'de> Visitor<'de> for IdentifierVisitor {
            type Value = RelativePathBuf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("relative path")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                RelativePathBuf::try_from(v)
                    .map_err(|err| serde::de::Error::custom(format!("{err}")))
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                RelativePathBuf::try_from(v.as_str())
                    .map_err(|err| serde::de::Error::custom(format!("{err}")))
            }
        }

        deserializer.deserialize_str(IdentifierVisitor)
    }
}

impl Serialize for RelativePathBuf {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
