//! Legacy v4 eventfiles folder structure support

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::event_entry::EventEntryV4;
use crate::{events_git, EventEntry};

enum HasChanged {
    Changed,
    Unchanged,
}

const FOLDER: &str = "eventfiles";
pub fn update() {
    events_git::checkout();
    let events = read_all_events().expect("Should be able to read eventfiles");
    fs::create_dir_all(FOLDER).expect("should be able to create the eventfiles folder");
    save_events(events);
}

fn read_all_events() -> anyhow::Result<Vec<EventEntry>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(events_git::FOLDER)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name
            .to_str()
            .expect("events should only contain unicode filenames");
        if file_name.starts_with('.') {
            continue;
        }
        result.append(&mut read_events_from_base(&entry.path())?);
    }
    Ok(result)
}

fn read_events_from_base(base_path: &Path) -> anyhow::Result<Vec<EventEntry>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(base_path)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "json");
        if !is_json {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let mut events = serde_json::from_str(&content)?;
        result.append(&mut events);
    }
    Ok(result)
}

fn save_events(all: Vec<EventEntry>) {
    let grouped = get_grouped(all);
    println!("Events by name: {}", grouped.len());

    let mut all_events = Vec::with_capacity(grouped.len());
    let mut expected_files = Vec::with_capacity(grouped.len());
    let mut changed_events = Vec::new();

    #[allow(clippy::iter_over_hash_type)]
    for (key, events) in grouped {
        all_events.push(events.first().unwrap().name.clone());
        let has_changed = save_events_to_file(&key, events);
        if matches!(has_changed, HasChanged::Changed) {
            changed_events.push(key.clone());
        }
        expected_files.push(key);
    }

    all_events.sort();
    let all_txt_content = all_events.join("\n") + "\n";
    drop(all_events);
    write_when_different("all.txt", &all_txt_content).expect("write all.txt");

    if !changed_events.is_empty() {
        changed_events.sort();
        println!("changed {} {changed_events:?}", changed_events.len());
    }

    let mut removed_events = cleanup_superfluous_eventfiles(&expected_files);
    if !removed_events.is_empty() {
        removed_events.sort();
        println!("deleted {} {removed_events:?}", removed_events.len());
    }
}

#[allow(clippy::min_ident_chars)]
/// Groups the events by their file name
fn get_grouped(all: Vec<EventEntry>) -> HashMap<String, Vec<EventEntry>> {
    fn ne<T: Ord>(a: &T, b: &T) -> Option<Ordering> {
        match a.cmp(b) {
            Ordering::Equal => None,
            other => Some(other),
        }
    }

    let mut grouped: HashMap<String, Vec<EventEntry>> = HashMap::new();
    for entry in all {
        let filename = entry.name.replace('/', "-");
        grouped.entry(filename).or_default().push(entry);
    }

    #[allow(clippy::iter_over_hash_type)]
    for groupvalues in grouped.values_mut() {
        groupvalues.sort_by(|a, b| {
            ne(&a.start, &b.start)
                .or_else(|| ne(&a.end, &b.end))
                .or_else(|| ne(&a.location, &b.location))
                .or_else(|| ne(&a.description, &b.description))
                .unwrap_or_else(|| a.name.cmp(&b.name))
        });
        groupvalues.dedup();
    }

    grouped
}

fn save_events_to_file(name: &str, events: Vec<EventEntry>) -> HasChanged {
    let filename = format!("{name}.json");
    let v4 = events
        .into_iter()
        .map(EventEntryV4::from)
        .collect::<Vec<_>>();
    let json = serde_json::to_string_pretty(&v4).expect("serialize events to json");
    write_when_different(&filename, &json).expect("write event file")
}

fn write_when_different(filename: &str, content: &str) -> std::io::Result<HasChanged> {
    let path = Path::new(FOLDER).join(filename);
    if fs::read_to_string(&path).is_ok_and(|current| current == content) {
        return Ok(HasChanged::Unchanged);
    }
    fs::write(&path, content)?;
    Ok(HasChanged::Changed)
}

fn cleanup_superfluous_eventfiles(expected_files: &[String]) -> Vec<String> {
    let mut removed = Vec::new();
    for maybe_entry in fs::read_dir(FOLDER).expect("should be able to read event file directory") {
        let path = maybe_entry
            .expect("should be able to inspect event file")
            .path();
        let is_json = path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        if !is_json {
            continue;
        }
        let Some(name) = path.file_stem().and_then(std::ffi::OsStr::to_str) else {
            eprintln!("WARNING: deleting non UTF8 named json file {path:?}");
            fs::remove_file(&path).expect("should be able to remove superfluous event file");
            continue;
        };
        let name = name.to_owned();
        if expected_files.contains(&name) {
            continue;
        }
        fs::remove_file(&path).expect("should be able to remove superfluous event file");
        removed.push(name);
    }
    removed
}
