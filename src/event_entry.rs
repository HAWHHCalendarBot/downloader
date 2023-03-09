use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    #[serde(serialize_with = "serialize_date_time")]
    pub start_time: NaiveDateTime,
    #[serde(serialize_with = "serialize_date_time")]
    pub end_time: NaiveDateTime,
}

fn serialize_date_time<S>(dt: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&Berlin.from_local_datetime(dt).unwrap().to_rfc3339())
}
