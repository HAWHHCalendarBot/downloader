use std::io::Read as _;
use std::sync::LazyLock;

use anyhow::Context as _;
use ureq::{Agent, Request};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

fn get_with_headers(url: &str) -> Request {
    static AGENT: LazyLock<Agent> =
        LazyLock::new(|| ureq::AgentBuilder::new().user_agent(USER_AGENT).build());

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
        .with_context(|| format!("read bytes from body {url}"))?;

    // Also known as ISO-8859-1 but that doesnt seem to be a defined standard.
    // https://docs.rs/encoding_rs/0.8.35/encoding_rs/index.html#iso-8859-1
    let haw_text = encoding_rs::mem::decode_latin1(&bytes).into_owned();

    Ok(haw_text)
}
