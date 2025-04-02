// Import various components for working with HTTP headers using reqwest.
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER};
// - HeaderMap: a type representing a collection of HTTP headers.
// - HeaderValue: represents a single HTTP header value.
// - USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER: constants for common HTTP header names.

// Import the prelude from the rand crate to bring commonly used traits (like SliceRandom)
// into scope for random number generation and selection from collections.
use rand::prelude::*;

// Import ThreadRng, the thread-local random number generator, which is used to generate random values
// in a thread-safe manner.
use rand::rngs::ThreadRng;

// Import the standard error trait, allowing functions to return errors of any type that implements Error.
use std::error::Error;

// Import the `spoof_ua` function from the ua_generator crate, which is used to generate a fake
// User-Agent string for HTTP requests.
use ua_generator::ua::spoof_ua;

pub fn generate_random_headers(url: &str) -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();

    // Random User-Agent using our existing UA generator.
    let user_agent = spoof_ua();
    headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent)?);

    // Random Accept header choices.
    let accept_options = [
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "text/html,application/xhtml+xml,application/xml;q=0.8,image/webp,*/*;q=0.6",
    ];
    let mut rng: ThreadRng = rand::rng();
    let accept = accept_options.choose(&mut rng).unwrap();
    headers.insert(ACCEPT, HeaderValue::from_static(accept));

    let lang_options = [
        "en-US,en;q=0.9",
        "en-GB,en;q=0.8",
        "en-US,en;q=0.9,fr;q=0.8,de;q=0.7",
        "es-ES,es;q=0.9,en;q=0.8",
        "fr-FR,fr;q=0.9,en;q=0.8",
        "de-DE,de;q=0.9,en;q=0.8",
        "it-IT,it;q=0.9,en;q=0.8",
        "nl-NL,nl;q=0.9,en;q=0.8",
        "pt-PT,pt;q=0.9,en;q=0.8",
    ];
    let lang = lang_options.choose(&mut rng).unwrap();
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static(lang));

    // Set Referer to the URL's domain.
    let domain = url.split('/').take(3).collect::<Vec<_>>().join("/");
    headers.insert(REFERER, HeaderValue::from_str(&domain)?);

    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Cache-Control", HeaderValue::from_static("max-age=0"));

    Ok(headers)
}
