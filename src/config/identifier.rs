use anyhow::{Context, Result};
use clap::builder::ValueParserFactory;
use serde::{
    de::{Deserialize, Deserializer, Visitor},
    ser::Serialize,
};

#[derive(Debug, Clone)]
pub struct Identifier(String);

impl Identifier {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Identifier {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        if value.is_empty() {
            anyhow::bail!("cannot create Identifier: source string is empty")
        }

        if let Some(invalid_char) = value
            .chars()
            .find(|ch| !(ch.is_ascii_alphanumeric() || ch == &'-' || ch == &'_'))
        {
            anyhow::bail!(
                "cannot create Identifier: source string contains invalid character `{invalid_char}`"
            )
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for Identifier {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        Self::try_from(value.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierVisitor;

        impl<'de> Visitor<'de> for IdentifierVisitor {
            type Value = Identifier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("identifer string (a-z, -, _)")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Identifier::try_from(v).map_err(|err| serde::de::Error::custom(format!("{err}")))
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Identifier::try_from(v).map_err(|err| serde::de::Error::custom(format!("{err}")))
            }
        }

        deserializer.deserialize_str(IdentifierVisitor)
    }
}

impl ValueParserFactory for Identifier {
    type Parser = fn(&str) -> Result<Identifier>;
    fn value_parser() -> Self::Parser {
        validate_identifier
    }
}

fn validate_identifier(value: &str) -> Result<Identifier> {
    Identifier::try_from(value).context("invalid character in identifier string")
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
