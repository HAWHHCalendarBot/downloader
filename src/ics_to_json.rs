use crate::event::EventEntry;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use regex::Regex;

const EVENT_REGEX: &str = r#"BEGIN:VEVENT\nSUMMARY:(.+)\nLOCATION:(.+)\n(?:DESCRIPTION:(.*)\n)?UID:(.+)\nDTSTART;TZID=Europe/Berlin:(.+)\nDTEND;TZID=Europe/Berlin:(.+)\nEND:VEVENT"#;
const LOCATION_REGEX: &str = r#"  Stand.+"#;

pub fn parse(ics_bodies: &[String]) -> Result<Vec<EventEntry>, String> {
    let mut all: Vec<EventEntry> = Vec::new();
    let event_regex = Regex::new(EVENT_REGEX).expect("Could not create ics regex");
    let location_regex = Regex::new(LOCATION_REGEX).expect("Could not create location regex");

    for body in ics_bodies {
        let mut one = parse_one(&event_regex, &location_regex, &body)?;
        all.append(&mut one);
    }

    Ok(all)
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
        let _uid = &cap[4];

        let description = parse_description(dozent);

        result.push(EventEntry {
            name: cap[1].trim().to_owned(),
            location: parse_location(&location_regex, &cap[2].trim()),
            description: description.to_owned(),
            start_time: parse_datetime(&cap[5].trim())?,
            end_time: parse_datetime(&cap[6].trim())?,
        });
    }

    Ok(result)
}

fn parse_datetime(raw: &str) -> Result<String, String> {
    let tless = raw.replace('T', " ");
    let naive = NaiveDateTime::parse_from_str(&tless, "%Y%m%d %H%M%S")
        .map_err(|err| format!("parse_datetime failed {} {}", raw, err))?;
    let date_time: DateTime<Local> = Local.from_local_datetime(&naive).unwrap();
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
    location_regex.replace_all(raw, "").to_string()
}
