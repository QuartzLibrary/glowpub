//! A small module to (de)serialise the glowfic timestamps directly into [chrono::Datetime<Utc>].

use std::fmt;

use chrono::{DateTime, Utc};

pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_str(Rfc3339Visitor)
}

struct Rfc3339Visitor;
impl<'de> serde::de::Visitor<'de> for Rfc3339Visitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a timestamp conforming to RFC3339")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        DateTime::parse_from_rfc3339(v)
            .map(From::from)
            .map_err(E::custom)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(std::str::from_utf8(v).map_err(E::custom)?)
    }
}
