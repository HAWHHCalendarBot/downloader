use std::io::Read;

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

    let content = ISO_8859_1
        .decode(&bytes, DecoderTrap::Replace)
        .expect("Decoder Trap cant fail");
    Ok(content)
}
