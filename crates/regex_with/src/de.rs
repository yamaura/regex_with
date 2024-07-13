use crate::capturable::Capturable;
use serde::de::value::MapDeserializer;
use serde::de::{self, Visitor};

pub use regex_with_derive::DeFromStr as FromStr;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub enum Error {
    #[error("no match")]
    NoMatch,
    Plain(#[from] serde_plain::Error),
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

pub struct Deserializer<'de, C: Capturable> {
    input: &'de str,
    capture: C,
}

impl<'de, C: Capturable> Deserializer<'de, C> {
    pub fn new(input: &'de str, capture: C) -> Self {
        Deserializer { input, capture }
    }
}

impl<'de, 'a, C: Capturable> de::Deserializer<'de> for &'a mut Deserializer<'de, C> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        //unimplemented!()
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (names, captures) = self.capture.captures(self.input).ok_or(Error::NoMatch)?;

        let items = names.filter_map(|n| {
            n.and_then(|name| {
                captures
                    .name(name)
                    .map(|value| (name.to_owned(), Value::new(value.as_str())))
            })
        });

        Ok(visitor.visit_map(MapDeserializer::new(items))?)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct(name, variants, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct /*enum*/ identifier ignored_any
    }
}

struct Value<'a>(serde_plain::Deserializer<'a>);

impl<'a> Value<'a> {
    fn new(input: &'a str) -> Value<'a> {
        Value(serde_plain::Deserializer::new(input))
    }
}

impl<'de> serde::de::IntoDeserializer<'de, serde_plain::Error> for Value<'de> {
    type Deserializer = serde_plain::Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.0
    }
}
