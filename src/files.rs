use std::collections::HashMap;
use std::fs::DirBuilder;
use std::path::Path;
use std::{fs, io};

use chrono::DateTime;
use itertools::Itertools;

use crate::EventEntry;

enum HasChanged {
    Changed,
    Unchanged,
}

const FOLDER: &str = "eventfiles";

pub fn ensure_folders_exist() -> Result<(), io::Error> {
    let mut dir = DirBuilder::new();
    dir.recursive(true).create(FOLDER)
}

pub fn save_events(all: &[EventEntry]) {
    let mut all_events: Vec<String> = Vec::new();
    let mut expected_files: Vec<String> = Vec::new();
    let mut changed_events: Vec<String> = Vec::new();
    let mut removed_events: Vec<String> = Vec::new();

    let grouped = get_grouped(all);
    println!("Events by name: {}", grouped.len());
    for key in grouped.keys() {
        let events = grouped.get(key).unwrap();
        let has_changed = save_events_to_file(key, events);
        if matches!(has_changed, HasChanged::Changed) {
            changed_events.push(key.clone());
        }

        all_events.push(key.clone());
        expected_files.push(key.replace('/', "-"));
    }

    all_events.sort();

    let all_txt_content = all_events.join("\n") + "\n";
    write_only_changed("all.txt", &all_txt_content).expect("failed to write all.txt");

    for existing_file in read_existing_eventfiles().unwrap() {
        if !expected_files.contains(&existing_file) {
            let filename = format!("{existing_file}.json");
            let path = Path::new(FOLDER).join(filename);
            fs::remove_file(path).expect("failed removing superflous event file");
            removed_events.push(existing_file);
        }
    }

    if !changed_events.is_empty() {
        changed_events.sort();
        println!("changed {} {changed_events:?}", changed_events.len());
    }

    if !removed_events.is_empty() {
        removed_events.sort();
        println!("deleted {} {removed_events:?}", removed_events.len());
    }
}

fn get_grouped(all: &[EventEntry]) -> HashMap<String, Vec<EventEntry>> {
    let mut grouped: HashMap<String, Vec<EventEntry>> = HashMap::new();

    for (key, group) in &all.iter().group_by(|o| o.name.clone()) {
        if !grouped.contains_key(&key) {
            grouped.insert(key.clone(), Vec::new());
        }

        let existing = grouped.get_mut(&key).unwrap();

        for entry in group {
            let json = serde_json::to_string(&entry).unwrap();
            let element = serde_json::from_str(&json).unwrap();
            existing.push(element);
        }
    }

    for group in grouped.values_mut() {
        group.sort_by_cached_key(|o| {
            DateTime::parse_from_rfc3339(&o.start_time).expect("starttime is not a valid datetime")
        });
        group.dedup_by_key(|o| serde_json::to_string(&o).unwrap());
    }

    grouped
}

fn save_events_to_file(name: &str, events: &[EventEntry]) -> HasChanged {
    let filename = format!("{}.json", name.replace('/', "-"));
    let json =
        serde_json::to_string_pretty(&events).expect("could not serialize the events to json");

    write_only_changed(&filename, &json).expect("failed to write file")
}

fn write_only_changed(filename: &str, content: &str) -> Result<HasChanged, io::Error> {
    let path = Path::new(FOLDER).join(filename);
    if let Ok(current) = fs::read_to_string(&path) {
        if current == content {
            return Ok(HasChanged::Unchanged);
        }
    }

    fs::write(&path, content.as_bytes())?;

    Ok(HasChanged::Changed)
}

fn read_existing_eventfiles() -> Result<Vec<String>, io::Error> {
    let mut list: Vec<String> = Vec::new();
    for maybe_entry in fs::read_dir(FOLDER)? {
        let filename = maybe_entry?
            .file_name()
            .into_string()
            .expect("filename contains something that can not be read easily with rust");

        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if filename.to_lowercase().ends_with(".json") {
            let len = filename.len();
            let without_extension = &filename[..len - 4 - 1];
            list.push(without_extension.to_owned());
        }
    }

    Ok(list)
}
