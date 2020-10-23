use crate::event::EventEntry;
use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::{fs, io};

#[derive(Serialize, Deserialize, Debug)]
pub struct AdditionalEvent {
    pub name: String,
    pub room: String,
    pub date: u8,
    pub month: u8,
    pub year: u16,
    pub starttime: String,
    pub endtime: String,
}

const FOLDER_GIT: &str = "additionalEventsGithub";

#[allow(clippy::non_ascii_literal)]
const DESCRIPTION: &str = "Dies ist eine zusätzliche, inoffizielle Veranstaltung: https://github.com/HAWHHCalendarBot/AdditionalEvents";

pub fn get() -> Result<Vec<EventEntry>, String> {
    pull()?;

    let files = get_filenames()
        .map_err(|err| format!("failed to read additional event directory {}", err))?;
    println!("Additionals: found {} event files", files.len());

    let mut events: Vec<EventEntry> = Vec::new();
    for file in files {
        let mut file_events = get_file(&file).map_err(|err| format!("failed! {}", err))?;
        events.append(&mut file_events);
    }

    Ok(events)
}

fn pull() -> Result<(), String> {
    let status = if Path::new("additionalEventsGithub/.git").exists() {
        Command::new("git")
            .arg("pull")
            .current_dir("additionalEventsGithub")
            .status()
            .map_err(|err| format!("failed to pull additional event repo {}", err))?
    } else {
        Command::new("git")
            .args(&[
                "clone",
                "-q",
                "--depth",
                "1",
                "https://github.com/HAWHHCalendarBot/AdditionalEvents",
                "additionalEventsGithub",
            ])
            .status()
            .map_err(|err| format!("failed to clone additional event repo {}", err))?
    };

    if status.success() {
        Ok(())
    } else {
        let error_message = format!("failed to clone/pull. Status code {}", status);
        Err(error_message)
    }
}

fn get_filenames() -> Result<Vec<String>, io::Error> {
    let mut list: Vec<String> = Vec::new();
    for maybe_entry in fs::read_dir("additionalEventsGithub/events")? {
        let filename = maybe_entry?
            .file_name()
            .into_string()
            .expect("filename contains something that can not be read easily with rust");

        if filename.ends_with(".json") {
            list.push(filename.to_owned());
        }
    }

    Ok(list)
}

fn get_file(name: &str) -> Result<Vec<EventEntry>, String> {
    let path = Path::new(FOLDER_GIT).join("events").join(name);
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read event file {} {}", name, err))?;
    let additionals: Vec<AdditionalEvent> = serde_json::from_str(&content)
        .map_err(|err| format!("failed to parse event file {} {}", name, err))?;

    let mut events: Vec<EventEntry> = Vec::with_capacity(additionals.capacity());
    for additional in additionals {
        events.push(parse_additional_event_to_event_entry(&additional)?);
    }

    Ok(events)
}

fn parse_additional_event_to_event_entry(before: &AdditionalEvent) -> Result<EventEntry, String> {
    let start = parse_datetime(before.year, before.month, before.date, &before.starttime)?;
    let end = parse_datetime(before.year, before.month, before.date, &before.endtime)?;
    Ok(EventEntry {
        name: before.name.to_owned(),
        location: before.room.to_owned(),
        description: DESCRIPTION.to_owned(),
        start_time: start,
        end_time: end,
    })
}

fn parse_datetime(year: u16, month: u8, day: u8, time: &str) -> Result<String, String> {
    let string = format!("{} {} {} {}", year, month, day, time);
    let naive = NaiveDateTime::parse_from_str(&string, "%Y %m %d %H:%M")
        .map_err(|err| format!("parse_datetime failed {} {}", string, err))?;
    let date_time = Berlin.from_local_datetime(&naive).unwrap();
    Ok(date_time.to_rfc3339())
}

#[test]
fn can_parse_datetime() -> Result<(), String> {
    assert_eq!(
        "2020-12-04T22:04:00+01:00",
        parse_datetime(2020, 12, 4, "22:04")?
    );
    Ok(())
}
