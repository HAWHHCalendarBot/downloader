use std::io::Read;

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use ureq::{Agent, Request};

fn get_with_headers(agent: &Agent, url: &str) -> Request {
    agent
        .get(url)
        .set("user-agent", "github.com/HAWHHCalendarBot/downloader")
        .set("from", "calendarbot@hawhh.de")
}

pub fn get_text(agent: &Agent, url: &str) -> Result<String, String> {
    get_with_headers(agent, url)
        .call()
        .map_err(|err| format!("failed to get {} {}", url, err))?
        .into_string()
        .map_err(|err| format!("failed to read string {} {}", url, err))
}

pub fn get_haw_text(agent: &Agent, url: &str) -> Result<String, String> {
    let mut bytes: Vec<u8> = vec![];

    get_with_headers(agent, url)
        .call()
        .map_err(|err| format!("failed to get {} {}", url, err))?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("failed to read bytes {} {}", url, err))?;

    ISO_8859_1
        .decode(&bytes, DecoderTrap::Replace)
        .map_err(|err| format!("failed to parse encoding {} {}", url, err))
}
