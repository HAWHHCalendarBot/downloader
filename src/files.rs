use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::fs::DirBuilder;
use std::path::Path;

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
    let grouped = get_grouped(all);
    println!("Events by name: {}", grouped.len());

    let mut all_events = Vec::with_capacity(grouped.len());
    let mut expected_files = Vec::with_capacity(grouped.len());
    let mut changed_events = Vec::new();

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
fn get_grouped(all: Vec<EventEntry>) -> HashMap<String, Vec<EventEntry>> {
    fn ne<T: Ord>(a: &T, b: &T) -> Option<Ordering> {
        match a.cmp(b) {
            Ordering::Equal => None,
            other => Some(other),
        }
    }

    let mut grouped: HashMap<String, Vec<EventEntry>> = HashMap::new();
    for entry in all {
        grouped.entry(entry.name.clone()).or_default().push(entry);
    }

    for groupvalues in grouped.values_mut() {
        groupvalues.sort_by(|a, b| {
            ne(&a.start_time, &b.start_time)
                .or_else(|| ne(&a.end_time, &b.end_time))
                .or_else(|| ne(&a.location, &b.location))
                .unwrap_or_else(|| a.description.cmp(&b.description))
        });
        groupvalues.dedup();
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
                .map_or(false, |ext| ext.eq_ignore_ascii_case("json"));
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
