use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};
use std::collections::HashMap;

use crate::queue::Schedule;

const FORMAT: &'static str = "%Y-%m-%d";

pub fn serialize<S>(map: &HashMap<NaiveDate, Schedule>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let map = map
        .iter()
        .map(|(k, v)| (format!("{}", k.format(FORMAT)), v));
    serializer.collect_map(map)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<NaiveDate, Schedule>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = HashMap::<String, Schedule>::deserialize(deserializer)?;
    v.into_iter()
        .map(|(k, v)| {
            Ok((
                NaiveDate::parse_from_str(&k, FORMAT).map_err(serde::de::Error::custom)?,
                v,
            ))
        })
        .collect()
}
