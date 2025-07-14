use anyhow::{Error, Result};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;

// type for refresh rates
pub type RefreshRate = u32;

// type for serialized resolutions
pub type SerialiedResolution = String;

// type for parsed and usable resolution
#[derive(Clone, Hash, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

// implementation of resolution for serialization
impl Serialize for Resolution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_format = format!("{}x{}", self.width, self.height);
        serializer.serialize_str(&str_format)
    }
}

// Visitor to help deserialize the resolution from a string
struct ResolutionVisitor;

impl<'de> Visitor<'de> for ResolutionVisitor {
    type Value = Resolution;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string with the format `{width}x{height}`")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let options: Vec<&str> = v.split("x").collect();
        let width = options.get(0).ok_or(E::custom("unable to find width"))?;
        let height = options
            .get(1)
            .ok_or(E::custom("unable to extract height"))?;

        let width = width
            .parse::<u32>()
            .or(Err(E::custom("unable to parse width to u32")))?;
        let height = height
            .parse::<u32>()
            .or(Err(E::custom("unable to parse height to u32")))?;

        Ok(Resolution { width, height })
    }
}

impl<'de> Deserialize<'de> for Resolution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ResolutionVisitor)
    }
}

// implementation of equal and comparison traits for Resolution
impl Eq for Resolution {}
impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.width, self.height).cmp(&(other.width, other.height))
    }
}
impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
