use std::thread::sleep;
use std::time::Duration;

use crate::event_entry::EventEntry;

mod additionals;
mod event_entry;
mod files;
mod http;
mod ics_to_json;
mod ics_urls;

const SLEEP_DURATION: Duration = Duration::from_secs(100 * 60); // 100 minutes
#[allow(dead_code)]
const WAIT_BETWEEEN_REQUESTS: Duration = Duration::from_millis(200); // 200 milliseconds

fn main() {
    files::ensure_folders_exist().expect("create folders");

    let mut error_count = 0;

    loop {
        println!("Its time for another download... Start!");
        match the_loop() {
            Ok(()) => {
                error_count = 0;
                println!("download successful");
            }
            Err(err) => {
                println!("download failed... {err:#}");
                error_count += 1;
                assert!(error_count <= 3, "too many download errors");
            }
        }

        println!("Wait till next download...\n\n");
        sleep(SLEEP_DURATION);
    }
}

fn the_loop() -> anyhow::Result<()> {
    let mut all_events = Vec::new();

    all_events.append(&mut part_ics());
    all_events.append(&mut additionals::get()?);

    files::save_events(all_events);
    Ok(())
}

fn part_ics() -> Vec<EventEntry> {
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn one_url(url: &str) -> anyhow::Result<Vec<EventEntry>> {
        if url.ends_with(".ics") {
            let content = http::get_haw_text(url)?;
            let entries = ics_to_json::parse(&content)?;
            Ok(entries)
        } else {
            unimplemented!("Extension not supported");
        }
    }

    let urls = ics_urls::get_all();
    let url_amount = urls.len();
    println!("ICS total urls: {url_amount}");

    let mut entries = Vec::new();
    let mut successful: usize = 0;

    #[cfg(debug_assertions)]
    let mut current: usize = 0;

    for url in urls {
        match one_url(url.as_str()) {
            Ok(mut one) => {
                entries.append(&mut one);
                successful += 1;
            }
            Err(err) => println!("WARNING: skip ics file url {url} {err:#}"),
        }

        #[cfg(debug_assertions)]
        {
            current += 1;
            if current % 25 == 0 {
                println!("ICS file download {current:4}/{url_amount}");
            }
        }

        #[cfg(not(debug_assertions))]
        sleep(WAIT_BETWEEEN_REQUESTS);
    }
    println!(
        "ICS downloaded {successful} urls with {} events",
        entries.len()
    );
    entries
}
