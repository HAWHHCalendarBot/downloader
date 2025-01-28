use std::sync::LazyLock;

use ureq::http::header::FROM;
use ureq::typestate::WithoutBody;
use ureq::{Agent, RequestBuilder};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

fn get_with_headers(url: &str) -> RequestBuilder<WithoutBody> {
    static AGENT: LazyLock<Agent> = LazyLock::new(|| {
        Agent::new_with_config(Agent::config_builder().user_agent(USER_AGENT).build())
    });
    AGENT
        .get(url)
        .header(FROM, "calendarbot-downloader@hawhh.de")
}

pub fn get_text(url: &str) -> Result<String, ureq::Error> {
    get_with_headers(url).call()?.into_body().read_to_string()
}

pub fn get_haw_text(url: &str) -> Result<String, ureq::Error> {
    let bytes = get_with_headers(url).call()?.into_body().read_to_vec()?;

    // Also known as ISO-8859-1 but that doesnt seem to be a defined standard.
    // https://docs.rs/encoding_rs/0.8.35/encoding_rs/index.html#iso-8859-1
    let haw_text = encoding_rs::mem::decode_latin1(&bytes).into_owned();

    Ok(haw_text)
}
