use ureq::http::header::{FROM, USER_AGENT};
use ureq::http::HeaderValue;
use ureq::typestate::WithoutBody;
use ureq::RequestBuilder;

const FROM_VALUE: &str = "calendarbot-downloader@hawhh.de";
const USER_AGENT_VALUE: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

fn get_with_headers(url: &str) -> RequestBuilder<WithoutBody> {
    ureq::get(url)
        .header(FROM, HeaderValue::from_static(FROM_VALUE))
        .header(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE))
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
