use lazy_regex::{lazy_regex, Regex};
use once_cell::sync::Lazy;
use url::Url;

use crate::http::get_text;

pub fn get_all() -> Vec<Url> {
    static SOURCES: Lazy<[Url; 4]> = Lazy::new(|| {
        [
            "https://userdoc.informatik.haw-hamburg.de/doku.php?id=stundenplan:ics_public",
            "https://www.haw-hamburg.de/en/study/degree-courses-a-z/study-courses-in-detail/course/courses/show/information-engineering/Studierende/",
            "https://www.haw-hamburg.de/hochschule/technik-und-informatik/departments/informations-und-elektrotechnik/studium/studienorganisation/studienplaene/",
            "https://www.haw-hamburg.de/studium/studiengaenge-a-z/studiengaenge-detail/course/courses/show/elektrotechnik-und-informationstechnik/Studierende/",
        ].map(|url| Url::parse(url).unwrap())
    });

    let mut result = Vec::new();
    for url in &*SOURCES {
        match get_from_url(url) {
            Ok(mut urls) => result.append(&mut urls),
            Err(err) => println!("WARNING: skip base url {url} {err}"),
        }
    }
    result.sort();
    result.dedup();
    result
}

fn get_from_url(base_url: &Url) -> anyhow::Result<Vec<Url>> {
    let body = get_text(base_url.as_str())?;
    let urls = get_from_body(base_url, &body)?;
    if urls.is_empty() {
        println!("WARNING: no ics urls from url {base_url}");
    }
    Ok(urls)
}

fn get_from_body(base_url: &Url, body: &str) -> Result<Vec<Url>, url::ParseError> {
    static REGEX: Lazy<Regex> = lazy_regex!(r#"href="(\S+\.ics)""#);

    let mut result = Vec::new();
    for cap in REGEX.captures_iter(body) {
        let full_url = base_url.join(&cap[1])?;
        result.push(full_url);
    }
    Ok(result)
}
