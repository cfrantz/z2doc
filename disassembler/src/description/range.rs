use num_traits::identities::{one, zero};
use num_traits::Num;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Serialize, Deserialize)]
struct MyRangeInclusive<T: Num>(T, T);

impl<T: Num> Default for MyRangeInclusive<T> {
    fn default() -> Self {
        MyRangeInclusive(one(), zero())
    }
}

pub fn serialize<S, T>(r: &RangeInclusive<T>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + Copy + Num,
{
    let r = MyRangeInclusive(*r.start(), *r.end());
    r.serialize(ser)
}

pub fn deserialize<'de, D, T>(deser: D) -> Result<RangeInclusive<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Copy + Num,
{
    let r = MyRangeInclusive::deserialize(deser)?;
    Ok(RangeInclusive::new(r.0, r.1))
}
