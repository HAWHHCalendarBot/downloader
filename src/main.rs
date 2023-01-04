#![forbid(unsafe_code)]

use std::thread::sleep;
use std::time::Duration;

use crate::event::EventEntry;

mod additionals;
mod event;
mod files;
mod http;
mod ics_to_json;
mod ics_urls;

const SLEEP_DURATION: Duration = Duration::from_secs(100 * 60); // 100 minutes
#[allow(dead_code)]
const WAIT_BETWEEEN_REQUESTS: Duration = Duration::from_millis(200); // 200 milliseconds

fn main() {
    files::ensure_folders_exist().expect("failed to create folders");

    let mut error_count = 0;

    loop {
        println!("Its time for another download... Start!");
        match the_loop() {
            Ok(_) => {
                error_count = 0;
                println!("download successful");
            }
            Err(err) => {
                println!("download failed... {err}");
                error_count += 1;
                assert!(error_count <= 3, "too many download errors");
            }
        }

        println!("Wait till next download...\n\n");
        sleep(SLEEP_DURATION);
    }
}

fn the_loop() -> Result<(), String> {
    let mut all_events: Vec<EventEntry> = Vec::new();

    let mut ics_events = part_ics();
    println!("ICS events: {}", ics_events.len());
    all_events.append(&mut ics_events);

    let mut additional_events = additionals::get()?;
    println!("Additional events: {}", additional_events.len());
    all_events.append(&mut additional_events);

    files::save_events(&all_events);

    Ok(())
}

fn part_ics() -> Vec<EventEntry> {
    fn one_url(url: &str) -> Result<Vec<EventEntry>, String> {
        let content = http::get_haw_text(url)?;
        let entries = ics_to_json::parse(&content)?;
        Ok(entries)
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
            Err(err) => println!("WARNING: url will be skipped: {err}"),
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
    println!("ICS downloaded files: {successful}");

    entries
}
