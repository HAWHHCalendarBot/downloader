use crate::event::EventEntry;
use std::thread::sleep;
use std::time::Duration;

mod event;
mod files;
mod http;
mod ics_to_json;
mod ics_urls;

const SLEEP_DURATION: Duration = Duration::from_secs(100 * 60); // 100 minutes
#[allow(dead_code)]
const WAIT_BETWEEEN_REQUESTS: Duration = Duration::from_millis(200); // 200 milliseconds

fn main() {
    let client = http::init_client().expect("Failed to init http client");
    files::ensure_folders_exist().expect("failed to create folders");

    #[allow(clippy::never_loop)]
    #[allow(unreachable_code)]
    loop {
        println!("Its time for another download… Start!");
        if let Err(err) = the_loop(&client) {
            println!("download failed… {}", err)
        }

        #[cfg(debug_assertions)]
        break;

        println!("Wait till next download…");
        sleep(SLEEP_DURATION);
    }
}

fn the_loop(client: &reqwest::blocking::Client) -> Result<(), String> {
    let urls = ics_urls::get_all_ics_urls(&client)?;
    println!("total ICS urls: {}", urls.len());

    let mut contents: Vec<String> = Vec::new();
    for url in &urls {
        let content = http::get_haw_text(client, &url)
            .map_err(|err| format!("failed to load url {} {}", url, err))?;
        contents.push(content);

        #[cfg(debug_assertions)]
        println!("downloaded {:4}/{}", contents.len(), urls.len());

        #[cfg(not(debug_assertions))]
        sleep(WAIT_BETWEEEN_REQUESTS);
    }
    println!("downloaded ics urls: {}", contents.len());

    let all = ics_to_json::parse(&contents)?;
    println!("ics events: {}", all.len());

    files::save_events_to_files(&all);

    println!("download successful");
    files::confirm_successful_run();

    Ok(())
}
