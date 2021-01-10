use radix_fmt::radix_36;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use url::{ParseError, Url};

#[derive(Clone)]
pub enum ShortenerResult {
    Ok(Shortener),
    Err(String),
}

#[derive(Clone)]
pub enum URLResult {
    Ok(String),
    Err,
}

#[derive(Clone)]
pub struct Shortener {
    pub urls: Vec<String>,
}

fn read_urls<P>(filename: P) -> std::io::Result<String>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut output = String::new();
    file.read_to_string(&mut output);
    Ok(output)
}

impl Shortener {
    pub fn new(urls: Vec<String>) -> Shortener {
        Shortener { urls }
    }

    pub fn from_str(url_str: &str) -> Shortener {
        let urls: Vec<String> = url_str
            .split_terminator('\n')
            .map(|s| s.to_owned())
            .collect();

        Shortener::new(urls)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> ShortenerResult {
        match read_urls(path) {
            Ok(urls) => ShortenerResult::Ok(Shortener::from_str(&urls)),
            Err(err) => ShortenerResult::Err(format!("{}", err)),
        }
    }

    pub fn get_url(&self, short_code: &str) -> URLResult {
        if let Ok(index) = usize::from_str_radix(short_code, 36) {
            if index >= self.urls.len() {
                URLResult::Err
            } else {
                URLResult::Ok((&self.urls[index]).to_owned())
            }
        } else {
            URLResult::Err
        }
    }

    pub fn add_url(&mut self, url: &str) -> Result<String, ParseError> {
        match Url::parse(url) {
            Ok(parsed_url) => {
                if !parsed_url.has_authority() {
                    return Err(ParseError::RelativeUrlWithoutBase);
                }
                self.urls.push(url.to_owned());
                let short_code = format!("{}", radix_36(self.urls.len() - 1));
                Ok(short_code)
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(feature = "debug-output")]
    pub fn list_all(&self) -> String {
        self.urls
            .iter()
            .enumerate()
            .map(|(index, url)| format!("{}: {}\n", radix_36(index), url))
            .collect::<String>()
    }
}
