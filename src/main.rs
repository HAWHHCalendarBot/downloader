use std::fs;
use std::path::Path;

use url::Url;

use crate::event_entry::EventEntry;

mod event_entry;
mod events_git;
mod files;
mod http;
mod ics_to_json;
mod ics_urls;
mod v4;

fn main() {
    download_ics();

    println!("\n\n## Generate merged eventfiles compatible with downloader v4");
    v4::update();
}

fn download_ics() {
    events_git::pull();

    let base_urls = ics_urls::get_all();
    let url_amount = base_urls.values().flatten().count();
    println!("ICS total urls: {url_amount}");

    #[cfg(debug_assertions)]
    let mut current: usize = 0;
    let mut successful: usize = 0;

    #[expect(clippy::iter_over_hash_type)]
    for (base, urls) in base_urls {
        let path = Path::new(events_git::FOLDER).join(base);
        drop(fs::remove_dir_all(&path)); // Allowed to be empty
        fs::create_dir_all(&path).expect("create dir for base should work");
        for url in urls {
            match one_url(&path, &url) {
                Ok(()) => successful += 1,
                Err(err) => println!("WARNING: skip ics file url {url} {err:#}"),
            }

            #[cfg(debug_assertions)]
            {
                current += 1;
                if current % 25 == 0 {
                    println!("ICS file download {current:4}/{url_amount}");
                }
            }

            // Wait between requests
            #[cfg(not(debug_assertions))]
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        events_git::add(base);
    }

    println!("ICS downloaded {successful} urls");
    events_git::commit_and_push();
}

fn one_url(path: &Path, url: &Url) -> anyhow::Result<()> {
    let filestem = ics_urls::file_stem(url)?;
    let path = path.join(format!("{filestem}.json"));

    let ics_body = http::get_haw_text(url.as_str())?;
    let events = ics_to_json::parse(&ics_body)?;

    files::save_to_json(&path, &events);
    Ok(())
}
