use std::collections::HashMap;
use std::fs;
use std::fs::DirBuilder;
use std::path::Path;

use chrono::DateTime;

use crate::EventEntry;

enum HasChanged {
    Changed,
    Unchanged,
}

const FOLDER: &str = "eventfiles";

pub fn ensure_folders_exist() -> std::io::Result<()> {
    let mut dir = DirBuilder::new();
    dir.recursive(true).create(FOLDER)
}

pub fn save_events(all: Vec<EventEntry>) {
    let mut all_events: Vec<String> = Vec::new();
    let mut expected_files: Vec<String> = Vec::new();
    let mut changed_events: Vec<String> = Vec::new();
    let mut removed_events: Vec<String> = Vec::new();

    let grouped = get_grouped(all);
    println!("Events by name: {}", grouped.len());
    for (key, events) in grouped {
        let has_changed = save_events_to_file(&key, &events);
        if matches!(has_changed, HasChanged::Changed) {
            changed_events.push(key.clone());
        }
        expected_files.push(key.replace('/', "-"));
        all_events.push(key);
    }

    all_events.sort();

    let all_txt_content = all_events.join("\n") + "\n";
    write_when_different("all.txt", &all_txt_content).expect("write all.txt");

    for existing_file in read_existing_eventfiles().unwrap() {
        if !expected_files.contains(&existing_file) {
            let filename = format!("{existing_file}.json");
            let path = Path::new(FOLDER).join(filename);
            fs::remove_file(path).expect("removing superflous event file");
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

fn get_grouped(all: Vec<EventEntry>) -> HashMap<String, Vec<EventEntry>> {
    let mut grouped: HashMap<String, Vec<EventEntry>> = HashMap::new();
    for entry in all {
        grouped.entry(entry.name.clone()).or_default().push(entry);
    }

    for groupvalues in grouped.values_mut() {
        groupvalues.sort_by_cached_key(|o| {
            DateTime::parse_from_rfc3339(&o.start_time).expect("starttime is not a valid datetime")
        });
        groupvalues.dedup_by_key(|o| serde_json::to_string(&o).unwrap());
    }

    grouped
}

fn save_events_to_file(name: &str, events: &[EventEntry]) -> HasChanged {
    let filename = format!("{}.json", name.replace('/', "-"));
    let json = serde_json::to_string_pretty(events).expect("serialize events to json");
    write_when_different(&filename, &json).expect("write event file")
}

fn write_when_different(filename: &str, content: &str) -> std::io::Result<HasChanged> {
    let path = Path::new(FOLDER).join(filename);
    if let Ok(current) = fs::read_to_string(&path) {
        if current == content {
            return Ok(HasChanged::Unchanged);
        }
    }
    fs::write(&path, content)?;
    Ok(HasChanged::Changed)
}

fn read_existing_eventfiles() -> std::io::Result<Vec<String>> {
    let mut list: Vec<String> = Vec::new();
    for maybe_entry in fs::read_dir(FOLDER)? {
        let path = maybe_entry?.path();
        let is_json = path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("json"));
        if is_json {
            if let Some(name) = path.file_stem().and_then(std::ffi::OsStr::to_str) {
                list.push(name.to_string());
            }
        }
    }
    Ok(list)
}
