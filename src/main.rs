use crate::event::EventEntry;
use std::thread::sleep;
use std::time::Duration;

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

    #[allow(clippy::non_ascii_literal)]
    loop {
        println!("Its time for another download… Start!");
        if let Err(err) = the_loop() {
            println!("download failed… {}", err)
        }

        println!("Wait till next download…\n\n");
        sleep(SLEEP_DURATION);
    }
}

fn the_loop() -> Result<(), String> {
    let mut all_events: Vec<EventEntry> = Vec::new();

    let mut ics_events = part_ics()?;
    println!("ICS events: {}", ics_events.len());
    all_events.append(&mut ics_events);

    let mut additional_events = additionals::get()?;
    println!("Additional events: {}", additional_events.len());
    all_events.append(&mut additional_events);

    files::save_events(&all_events);

    println!("download successful");
    files::confirm_successful_run();

    Ok(())
}

fn part_ics() -> Result<Vec<EventEntry>, String> {
    let client = http::init_client().expect("Failed to init http client");
    let urls = ics_urls::get_all(&client)?;
    let url_amount = urls.len();
    println!("ICS total urls: {}", url_amount);

    let mut contents: Vec<String> = Vec::new();
    for url in urls {
        let content = http::get_haw_text(&client, &url)
            .map_err(|err| format!("failed to load url {} {}", url, err))?;
        contents.push(content);

        #[cfg(debug_assertions)]
        if contents.len() % 25 == 0 {
            println!("ICS file downloaded {:4}/{}", contents.len(), url_amount);
        }

        #[cfg(not(debug_assertions))]
        sleep(WAIT_BETWEEEN_REQUESTS);
    }
    println!("ICS downloaded files: {}", contents.len());

    ics_to_json::parse(&contents)
}
