use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use chrono::{NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Europe::Berlin;
use serde::{Deserialize, Serialize};

use crate::event_entry::EventEntry;

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

const DESCRIPTION: &str = "Dies ist eine zusätzliche, inoffizielle Veranstaltung: https://github.com/HAWHHCalendarBot/AdditionalEvents";

pub fn get() -> anyhow::Result<Vec<EventEntry>> {
    pull()?;

    let files = get_file_list().map_err(|err| anyhow!("read additional event directory {err}"))?;
    println!("Additionals: found {} event files", files.len());

    let mut events: Vec<EventEntry> = Vec::new();
    for file in files {
        let mut file_events = get_file(&file)
            .map_err(|err| anyhow!("Additionals file {:?} {err}", file.file_name()))?;
        events.append(&mut file_events);
    }
    println!("Additional events: {}", events.len());
    Ok(events)
}

fn pull() -> anyhow::Result<()> {
    let status = if Path::new("additionalEventsGithub/.git").exists() {
        Command::new("git")
            .arg("pull")
            .arg("--ff-only")
            .current_dir("additionalEventsGithub")
            .status()
            .map_err(|err| anyhow!("pull additional event repo {err}"))?
    } else {
        Command::new("git")
            .args([
                "clone",
                "-q",
                "--depth",
                "1",
                "https://github.com/HAWHHCalendarBot/AdditionalEvents",
                "additionalEventsGithub",
            ])
            .status()
            .map_err(|err| anyhow!("clone additional event repo {err}"))?
    };
    anyhow::ensure!(status.success(), "clone/pull. Status code {status}");
    Ok(())
}

fn get_file_list() -> std::io::Result<Vec<PathBuf>> {
    let mut list: Vec<PathBuf> = Vec::new();
    for maybe_entry in fs::read_dir("additionalEventsGithub/events")? {
        let path = maybe_entry?.path();
        let is_json = path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("json"));
        if is_json {
            list.push(path);
        }
    }

    Ok(list)
}

fn get_file(path: &Path) -> anyhow::Result<Vec<EventEntry>> {
    let content = fs::read_to_string(path)?;
    let additionals: Vec<AdditionalEvent> = serde_json::from_str(&content)?;

    let mut events: Vec<EventEntry> = Vec::with_capacity(additionals.capacity());
    for additional in additionals {
        events.push(parse_additional_event_to_event_entry(&additional)?);
    }

    Ok(events)
}

fn parse_additional_event_to_event_entry(before: &AdditionalEvent) -> anyhow::Result<EventEntry> {
    let start = parse_datetime(before.year, before.month, before.date, &before.starttime)?;
    let end = parse_datetime(before.year, before.month, before.date, &before.endtime)?;
    Ok(EventEntry {
        name: before.name.clone(),
        location: before.room.clone(),
        description: DESCRIPTION.to_owned(),
        start_time: start,
        end_time: end,
    })
}

fn parse_datetime(year: u16, month: u8, day: u8, time: &str) -> anyhow::Result<String> {
    let naive = NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
        .ok_or_else(|| anyhow!("parse_datetime day {year} {month} {day}"))?
        .and_time(
            NaiveTime::parse_from_str(time, "%H:%M")
                .map_err(|err| anyhow!("parse_datetime time {time} {err}"))?,
        );
    let date_time = Berlin.from_local_datetime(&naive).unwrap();
    Ok(date_time.to_rfc3339())
}

#[test]
fn can_parse_datetime() -> anyhow::Result<()> {
    assert_eq!(
        "2020-12-04T22:04:00+01:00",
        parse_datetime(2020, 12, 4, "22:04")?
    );
    Ok(())
}
