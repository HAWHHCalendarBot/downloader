use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::http::get_text;

pub fn get_all() -> Vec<Url> {
    static SOURCES: Lazy<[Url; 4]> = Lazy::new(|| {
        [
            "https://userdoc.informatik.haw-hamburg.de/doku.php?id=stundenplan:ics_public",
            "https://www.haw-hamburg.de/en/study/degree-courses-a-z/study-courses-in-detail/course/courses/show/information-engineering/Studierende/",
            "https://www.haw-hamburg.de/hochschule/technik-und-informatik/departments/informations-und-elektrotechnik/studium/studienorganisation/studienplaene/",
            "https://www.haw-hamburg.de/studium/studiengaenge-a-z/studiengaenge-detail/course/courses/show/elektrotechnik-und-informationstechnik/Studierende/",
        ].map(|u| Url::parse(u).unwrap())
    });

    let mut result: Vec<Url> = Vec::new();
    for url in SOURCES.iter() {
        match get_from_url(url) {
            Ok(mut urls) => result.append(&mut urls),
            Err(err) => println!("WARNING: skip base url {url} {err}"),
        }
    }
    result
}

fn get_from_url(base_url: &Url) -> Result<Vec<Url>, String> {
    let body = get_text(base_url.as_str())?;
    let urls =
        get_from_body(base_url, &body).map_err(|err| format!("parsing urls failed {err}"))?;
    if urls.is_empty() {
        println!("WARNING: no ics urls from url {base_url}");
    }
    Ok(urls)
}

fn get_from_body(base_url: &Url, body: &str) -> Result<Vec<Url>, url::ParseError> {
    static ICS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"href="(\S+\.ics)""#).unwrap());

    let mut result: Vec<Url> = Vec::new();
    for cap in ICS_REGEX.captures_iter(body) {
        let full_url = base_url.join(&cap[1])?;
        result.push(full_url);
    }
    Ok(result)
}
