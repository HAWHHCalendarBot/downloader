use chrono::{NaiveDateTime, TimeZone as _};
use chrono_tz::Europe::Berlin;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct EventEntryV4 {
    pub name: String,
    pub location: String,
    pub description: String,
    #[serde(serialize_with = "serialize_date_time")]
    pub start_time: NaiveDateTime,
    #[serde(serialize_with = "serialize_date_time")]
    pub end_time: NaiveDateTime,
}

fn serialize_date_time<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&Berlin.from_local_datetime(dt).unwrap().to_rfc3339())
}

impl From<EventEntry> for EventEntryV4 {
    fn from(value: EventEntry) -> Self {
        Self {
            name: value.name,
            location: value.location,
            description: value.description,
            start_time: value.start,
            end_time: value.end,
        }
    }
}
