use regex::Regex;
use ureq::Agent;
use url::Url;

use crate::http::get_text;

const ICS_REGEX: &str = r#"href="(\S+\.ics)""#;

const SOURCE_URLS: &[&str] = &[
    "https://userdoc.informatik.haw-hamburg.de/doku.php?id=stundenplan:ics_public",
    "https://www.haw-hamburg.de/en/study/degree-courses-a-z/study-courses-in-detail/course/courses/show/information-engineering/Studierende/",
    "https://www.haw-hamburg.de/studium/studiengaenge-a-z/studiengaenge-detail/course/courses/show/elektrotechnik-und-informationstechnik/Studierende/",
];

pub fn get_all(agent: &Agent) -> Result<Vec<String>, String> {
    let mut result: Vec<String> = Vec::new();

    for url in SOURCE_URLS {
        let mut urls = get_from_url(&agent, url)
            .map_err(|err| format!("failed to get ics urls from {} {}", url, err))?;
        result.append(&mut urls);
    }

    Ok(result)
}

fn get_from_url(agent: &Agent, base_url: &str) -> Result<Vec<String>, String> {
    let body = get_text(agent, base_url)?;

    let urls =
        get_from_body(base_url, &body).map_err(|err| format!("parsing urls failed {}", err))?;

    if urls.is_empty() {
        println!("WARNING: no ics urls from url {}", base_url);
    }

    Ok(urls)
}

fn get_from_body(base_url: &str, body: &str) -> Result<Vec<String>, url::ParseError> {
    let mut result: Vec<String> = Vec::new();

    let this_document = Url::parse(base_url)?;

    let re = Regex::new(ICS_REGEX).expect("Could not create ics regex");
    for cap in re.captures_iter(body) {
        let full_url = this_document.join(&cap[1])?;
        result.push(full_url.into_string());
    }

    Ok(result)
}
