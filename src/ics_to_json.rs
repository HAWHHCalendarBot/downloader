use anyhow::anyhow;
use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use lazy_regex::{lazy_regex, Lazy, Regex};

use crate::event_entry::EventEntry;

pub fn parse(ics_body: &str) -> anyhow::Result<Vec<EventEntry>> {
    static EVENT_REGEX: Lazy<Regex> = lazy_regex!(
        r#"BEGIN:VEVENT\nSUMMARY:(.+)\nLOCATION:(.+)\n(?:DESCRIPTION:(.*)\n)?UID:(.+)\nDTSTART;TZID=Europe/Berlin:(.+)\nDTEND;TZID=Europe/Berlin:(.+)\nEND:VEVENT"#
    );

    let mut result: Vec<EventEntry> = Vec::new();

    let sane_body = ics_body.replace("\r\n", "\n");

    for cap in EVENT_REGEX.captures_iter(&sane_body) {
        let dozent = cap[3].trim();
        // let uid = &cap[4];

        let description = parse_description(dozent);

        result.push(EventEntry {
            name: cap[1].trim().to_owned(),
            location: parse_location(cap[2].trim()),
            description,
            start_time: parse_datetime(cap[5].trim())?,
            end_time: parse_datetime(cap[6].trim())?,
        });
    }

    Ok(result)
}

fn parse_datetime(raw: &str) -> anyhow::Result<String> {
    let tless = raw.replace('T', " ");
    let naive = NaiveDateTime::parse_from_str(&tless, "%Y%m%d %H%M%S")
        .map_err(|err| anyhow!("parse_datetime {raw} {err}"))?;
    let date_time = Berlin.from_local_datetime(&naive).unwrap();
    // let nanos = date_time.timestamp_millis();
    // let offset = date_time.offset().to_string().replace(":", "");
    // let result = format!("/Date({nanos}{offset})/");
    Ok(date_time.to_rfc3339())
}

fn parse_description(dozent: &str) -> String {
    if dozent.is_empty() {
        String::new()
    } else {
        format!("Dozent: {dozent}")
    }
}

fn parse_location(raw: &str) -> String {
    static LOCATION_REGEX: Lazy<Regex> = lazy_regex!(r#"Stand \d{2}-\d{2}-\d{4}"#);

    LOCATION_REGEX.replace_all(raw, "").trim().to_string()
}

#[test]
fn can_parse_ics_datetime() -> anyhow::Result<()> {
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
    assert_eq!(
        "Stiftstr69 R304a",
        parse_location("Stiftstr69 R304a  Stand 12-03-2020")
    );
}

#[test]
fn location_being_only_stand_ends_up_empty() {
    assert_eq!("", parse_location("Stand 12-03-2020"));
}
