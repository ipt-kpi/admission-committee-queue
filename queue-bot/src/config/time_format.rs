use chrono::NaiveTime;
use serde::{Deserialize, Deserializer, Serializer};

const FORMAT: &'static str = "%H:%M:%S";

pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", time.format(FORMAT));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
}
