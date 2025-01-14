use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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

    let mut events = Vec::new();
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
                "https://github.com/HAWHHCalendarBot/AdditionalEvents.git",
                "additionalEventsGithub",
            ])
            .status()
            .map_err(|err| anyhow!("clone additional event repo {err}"))?
    };
    anyhow::ensure!(status.success(), "clone/pull. Status code {status}");
    Ok(())
}

fn get_file_list() -> std::io::Result<Vec<PathBuf>> {
    let mut list = Vec::new();
    for maybe_entry in fs::read_dir("additionalEventsGithub/events")? {
        let path = maybe_entry?.path();
        let is_json = path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        if is_json {
            list.push(path);
        }
    }
    Ok(list)
}

fn get_file(path: &Path) -> anyhow::Result<Vec<EventEntry>> {
    let content = fs::read_to_string(path)?;
    let additionals: Vec<AdditionalEvent> = serde_json::from_str(&content)?;
    let mut events = Vec::with_capacity(additionals.len());
    for additional in additionals {
        events.push(additional.into_event_entry()?);
    }
    Ok(events)
}

impl AdditionalEvent {
    fn into_event_entry(self) -> anyhow::Result<EventEntry> {
        let start = parse_datetime(self.year, self.month, self.date, &self.starttime)?;
        let end = parse_datetime(self.year, self.month, self.date, &self.endtime)?;
        Ok(EventEntry {
            name: self.name,
            location: self.room,
            description: DESCRIPTION.to_owned(),
            start_time: start,
            end_time: end,
        })
    }
}

fn parse_datetime(year: u16, month: u8, day: u8, time: &str) -> anyhow::Result<NaiveDateTime> {
    let naive = NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
        .ok_or_else(|| anyhow!("parse_datetime day {year} {month} {day}"))?
        .and_time(
            NaiveTime::parse_from_str(time, "%H:%M")
                .map_err(|err| anyhow!("parse_datetime time {time} {err}"))?,
        );
    Ok(naive)
}

#[test]
fn can_parse_datetime() -> anyhow::Result<()> {
    assert_eq!(
        NaiveDate::from_ymd_opt(2020, 12, 4)
            .unwrap()
            .and_hms_opt(22, 4, 0)
            .unwrap(),
        parse_datetime(2020, 12, 4, "22:04")?
    );
    Ok(())
}
