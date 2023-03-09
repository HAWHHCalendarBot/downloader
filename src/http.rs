use std::io::{Cursor, Read};

use anyhow::anyhow;
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use once_cell::sync::Lazy;
use ureq::{Agent, Request};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

fn get_with_headers(url: &str) -> Request {
    static AGENT: Lazy<Agent> =
        Lazy::new(|| ureq::AgentBuilder::new().user_agent(USER_AGENT).build());

    AGENT
        .get(url)
        .set("from", "calendarbot-downloader@hawhh.de")
}

pub fn get_text(url: &str) -> anyhow::Result<String> {
    let content = get_with_headers(url).call()?.into_string()?;
    Ok(content)
}

pub fn get_haw_text(url: &str) -> anyhow::Result<String> {
    let mut bytes: Vec<u8> = vec![];
    get_with_headers(url)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| anyhow!("read bytes from body {url} {err}"))?;
    Ok(decode_haw_text(&bytes))
}

pub fn get_ics_from_zip(url: &str) -> anyhow::Result<Vec<String>> {
    let mut bytes = Vec::new();

    get_with_headers(url)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| anyhow!("read bytes from body {url} {err}"))?;

    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)?;

    let has_ics_files = archive.file_names().any(|name| name.ends_with("ics"));
    anyhow::ensure!(has_ics_files, "no ics in zip");

    let mut result = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("within archive.len()");
        if !file.name().ends_with("ics") {
            eprintln!("WARNING: {url} contains non ics {}", file.name());
            continue;
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        result.push(decode_haw_text(&bytes));
    }
    Ok(result)
}

fn decode_haw_text(bytes: &[u8]) -> String {
    ISO_8859_1
        .decode(bytes, DecoderTrap::Replace)
        .expect("Decoder Trap cant fail")
}
