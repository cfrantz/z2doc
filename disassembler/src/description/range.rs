use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Default, Serialize, Deserialize)]
struct MyRange<T>(T, T);

pub fn serialize<S, T>(r: &Range<T>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + Copy,
{
    let r = MyRange(r.start, r.end);
    r.serialize(ser)
}

pub fn deserialize<'de, D, T>(deser: D) -> Result<Range<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Copy,
{
    let r = MyRange::deserialize(deser)?;
    Ok(Range {
        start: r.0,
        end: r.1,
    })
}
