use regex::Regex;
use url::Url;

const ICS_REGEX: &str = r#"href="(\S+\.ics)""#;

static SOURCE_URLS: &[&str] = &[
    "https://userdoc.informatik.haw-hamburg.de/doku.php?id=stundenplan:ics_public",
    "https://www.haw-hamburg.de/studium/studiengaenge-a-z/studiengaenge-detail/course/courses/show/elektrotechnik-und-informationstechnik/Studierende/"
];

pub fn get_all_ics_urls(client: &reqwest::blocking::Client) -> Result<Vec<String>, String> {
    let mut result: Vec<String> = Vec::new();

    for url in SOURCE_URLS {
        let mut urls = get_ics_urls_from_url(&client, url)
            .map_err(|err| format!("failed to get ics urls from {} {}", url, err))?;
        result.append(&mut urls);
    }

    Ok(result)
}

fn get_ics_urls_from_url(
    client: &reqwest::blocking::Client,
    base_url: &str,
) -> Result<Vec<String>, String> {
    let response = client
        .get(base_url)
        .send()
        .map_err(|err| format!("HTTP reqwest failed {}", err))?;

    let body = response
        .text()
        .map_err(|err| format!("reading text from reqwest failed {}", err))?;

    let urls = get_ics_urls_from_body(base_url, &body)
        .map_err(|err| format!("parsing urls failed {}", err))?;

    if urls.is_empty() {
        Err("no ics urls found".to_owned())
    } else {
        Ok(urls)
    }
}

fn get_ics_urls_from_body(base_url: &str, body: &str) -> Result<Vec<String>, url::ParseError> {
    let mut result: Vec<String> = Vec::new();

    let this_document = Url::parse(base_url)?;

    let re = Regex::new(ICS_REGEX).expect("Could not create ics regex");
    for cap in re.captures_iter(&body) {
        let full_url = this_document.join(&cap[1])?;
        result.push(full_url.into_string());
    }

    Ok(result)
}
