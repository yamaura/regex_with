#![doc = include_str!("../README.md")]
pub mod capturable;
pub mod de;

pub use de::Deserializer;
pub use regex_with_derive::Capturable;

pub use regex;
