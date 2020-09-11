use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use reqwest::blocking::Client;
use reqwest::{header, Error};

/// Create a http client
///
/// # Example
///
/// ```
/// let client = init_client()?;
/// let response = client.get(url).send()?;
/// let body = response.text()?;
/// ```
pub fn init_client() -> Result<Client, Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("github.com/HAWHHCalendarBot/downloader"),
    );
    headers.insert(
        header::FROM,
        header::HeaderValue::from_static("calendarbot@hawhh.de"),
    );
    Client::builder().default_headers(headers).build()
}

pub fn get_haw_text(client: &Client, url: &str) -> Result<String, String> {
    let response = client
        .get(url)
        .send()
        .map_err(|err| format!("failed to get {} {}", url, err))?;

    // Normally you would use response.text() but the haw had to use some non UTF8 encoding…

    let bytes = response
        .bytes()
        .map_err(|err| format!("failed to get bytes from response {} {}", url, err))?;

    ISO_8859_1
        .decode(&bytes, DecoderTrap::Replace)
        .map_err(|err| format!("failed to parse encoding {} {}", url, err))
}
