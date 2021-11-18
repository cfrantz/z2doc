use anyhow::{anyhow, Result};
use std::marker::PhantomData;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use void::Void;


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default, skip_serializing_if="String::is_empty")]
    pub comment: String,
    #[serde(default, skip_serializing_if="String::is_empty")]
    pub symbol: String,
    #[serde(default, skip_serializing_if="String::is_empty")]
    pub header: String,
    #[serde(default, skip_serializing_if="String::is_empty")]
    pub footer: String,
}

impl FromStr for Comment {
    type Err = Void;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Comment {
            comment: s.to_string(),
            ..Default::default()
        })
    }
}

pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
