//! Defines the node type.
use serde::{Deserialize, Serialize};

/// Defines the node type.
/// MUST BE either "anchor" or "non-anchor"
#[derive(
    Deserialize,
    Serialize,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub enum Kind {
    #[serde(rename = "anchor")]
    Anchor,
    #[serde(rename = "non-anchor")]
    NonAnchor,
    Unknown(String),
}

impl std::convert::From<&str> for Kind {
    fn from(s: &str) -> Self {
        match s {
            "anchor" => Kind::Anchor,
            "non-anchor" => Kind::NonAnchor,
            "non_anchor" => Kind::NonAnchor,

            other => Kind::Unknown(other.to_owned()),
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Kind::from(s))
    }
}

impl Kind {
    /// Returns the `&str` value of the enum member.
    pub fn as_str(&self) -> &str {
        match self {
            Kind::Anchor => "anchor",
            Kind::NonAnchor => "non-anchor",

            Kind::Unknown(s) => s.as_ref(),
        }
    }

    /// Returns all the `&str` values of the enum members.
    pub fn values() -> &'static [&'static str] {
        &[
            "anchor",     //
            "non-anchor", //
        ]
    }
}

impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
