use std::collections::HashMap;

use anyhow::Context as _;
use lazy_regex::regex;
use url::Url;

use crate::http::get_text;

pub fn get_all() -> HashMap<&'static str, Vec<Url>> {
    let sources = [
        ("informatik", "https://userdoc.informatik.haw-hamburg.de/doku.php?id=stundenplan:ics_public"),
        ("information-engineering", "https://www.haw-hamburg.de/en/study/degree-courses-a-z/study-courses-in-detail/course/courses/show/information-engineering/Studierende/"),

        // nearly the same but not identical
        ("elektrotechnik-und-informationstechnik", "https://www.haw-hamburg.de/studium/studiengaenge-a-z/studiengaenge-detail/course/courses/show/elektrotechnik-und-informationstechnik/Studierende/"),
        ("informations-und-elektrotechnik", "https://www.haw-hamburg.de/hochschule/technik-und-informatik/departments/informations-und-elektrotechnik/studium/studienorganisation/studienplaene/"),
    ];

    let mut result = HashMap::new();
    for (base, url) in sources {
        let mut below = Vec::new();
        let url = Url::parse(url).unwrap();
        match get_from_url(&url) {
            Ok(mut urls) => below.append(&mut urls),
            Err(err) => println!("WARNING: skip base url {url} {err:#}"),
        }
        below.sort();
        below.dedup();
        result.insert(base, below);
    }
    result
}

fn get_from_url(base_url: &Url) -> anyhow::Result<Vec<Url>> {
    let body = get_text(base_url.as_str())?;
    let urls = get_from_body(base_url, &body)?;
    anyhow::ensure!(!urls.is_empty(), "no ics urls found");
    Ok(urls)
}

fn get_from_body(base_url: &Url, body: &str) -> Result<Vec<Url>, url::ParseError> {
    let mut result = Vec::new();
    for cap in regex!(r#"href="(\S+\.ics)""#).captures_iter(body) {
        let full_url = base_url.join(&cap[1])?;
        result.push(full_url);
    }
    Ok(result)
}

/// As HAW is complicated, try to get the filestem of the file to be downloaded
pub fn file_stem(url: &Url) -> anyhow::Result<&str> {
    if url.path() == "/lib/exe/fetch.php" {
        return informatik_userdoc_fetch_file_stem(url).context("informatik userdoc fetch");
    }

    let filename = url
        .path_segments()
        .context("ICS url should always have a path")?
        .next_back()
        .context("path_segments should always return at least one segment")?;
    let (filestem, _) = filename
        .rsplit_once('.')
        .context("should always contain a .")?;
    Ok(filestem)
}

fn informatik_userdoc_fetch_file_stem(url: &Url) -> anyhow::Result<&str> {
    let query = url.query().context("should have a query")?;
    let capture = regex!(r"media=stundenplan:(.+)\.ics")
        .captures(query)
        .context("should match expected media query")?;
    let filestem = capture.get(1).unwrap().as_str();
    Ok(filestem)
}
