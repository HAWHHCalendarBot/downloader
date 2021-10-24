use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use regex::Regex;

use crate::event::EventEntry;

const EVENT_REGEX: &str = r#"BEGIN:VEVENT\nSUMMARY:(.+)\nLOCATION:(.+)\n(?:DESCRIPTION:(.*)\n)?UID:(.+)\nDTSTART;TZID=Europe/Berlin:(.+)\nDTEND;TZID=Europe/Berlin:(.+)\nEND:VEVENT"#;
const LOCATION_REGEX: &str = r#"Stand \d{2}-\d{2}-\d{4}"#;

pub struct IcsToJson {
    event_regex: Regex,
    location_regex: Regex,
}

impl IcsToJson {
    pub fn new() -> Self {
        Self {
            event_regex: Regex::new(EVENT_REGEX).expect("Could not create ics regex"),
            location_regex: Regex::new(LOCATION_REGEX).expect("Could not create location regex"),
        }
    }

    pub fn parse(&self, ics_body: &str) -> Result<Vec<EventEntry>, String> {
        parse_one(&self.event_regex, &self.location_regex, ics_body)
    }
}

fn parse_one(
    event_regex: &Regex,
    location_regex: &Regex,
    ics_body: &str,
) -> Result<Vec<EventEntry>, String> {
    let mut result: Vec<EventEntry> = Vec::new();

    let sane_body = ics_body.replace("\r\n", "\n");

    for cap in event_regex.captures_iter(&sane_body) {
        let dozent = cap[3].trim();
        // let uid = &cap[4];

        let description = parse_description(dozent);

        result.push(EventEntry {
            name: cap[1].trim().to_owned(),
            location: parse_location(location_regex, cap[2].trim()),
            description,
            start_time: parse_datetime(cap[5].trim())?,
            end_time: parse_datetime(cap[6].trim())?,
        });
    }

    Ok(result)
}

fn parse_datetime(raw: &str) -> Result<String, String> {
    let tless = raw.replace('T', " ");
    let naive = NaiveDateTime::parse_from_str(&tless, "%Y%m%d %H%M%S")
        .map_err(|err| format!("parse_datetime failed {} {}", raw, err))?;
    let date_time = Berlin.from_local_datetime(&naive).unwrap();
    // let nanos = date_time.timestamp_millis();
    // let offset = date_time.offset().to_string().replace(":", "");
    // let result = format!("/Date({}{})/", nanos, offset);
    Ok(date_time.to_rfc3339())
}

fn parse_description(dozent: &str) -> String {
    if dozent.is_empty() {
        "".to_owned()
    } else {
        format!("Dozent: {}", dozent)
    }
}

fn parse_location(location_regex: &Regex, raw: &str) -> String {
    location_regex.replace_all(raw, "").trim().to_string()
}

#[test]
fn can_parse_ics_datetime() -> Result<(), String> {
    assert_eq!(
        "2020-12-05T22:04:00+01:00",
        parse_datetime("20201205T220400")?
    );
    assert_eq!(
        "2020-07-05T12:04:00+02:00",
        parse_datetime("20200705T120400")?
    );
    Ok(())
}

#[test]
fn empty_dozent_ends_up_as_empty_description() {
    assert_eq!("", parse_description(""));
}

#[test]
fn some_dozent_ends_up_as_description() {
    assert_eq!("Dozent: HTM", parse_description("HTM"));
}

#[test]
fn location_gets_stand_removed() {
    let regex = Regex::new(LOCATION_REGEX).expect("failed to parse Regex");
    assert_eq!(
        "Stiftstr69 R304a",
        parse_location(&regex, "Stiftstr69 R304a  Stand 12-03-2020")
    );
}

#[test]
fn location_being_only_stand_ends_up_empty() {
    let regex = Regex::new(LOCATION_REGEX).expect("failed to parse Regex");
    assert_eq!("", parse_location(&regex, "Stand 12-03-2020"));
}
