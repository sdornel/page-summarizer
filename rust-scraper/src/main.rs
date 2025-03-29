
use std::fs; // file parsing
use std::error::Error; // allows us to use Result for error handling
use ua_generator::ua::spoof_ua; // fake user agent
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER}; // allows to set headers

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let json_data = fs::read_to_string("../output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;

    println!("Raw JSON data:\n{}", json_data);

    for (i, url) in urls.iter().enumerate() {
        if url.contains("duckduckgo.com/?q=") {
            println!("{} => ⏩ Skipped DuckDuckGo redirect: {}", i, url);
            continue;
        }

        println!("{} => {}", i, url);
        match fetch_url(url).await {
            Ok(body) => println!("✅ Success: {} bytes\n", body.len()),
            Err(error) => println!("❌ Failed to fetch: {}\n", error),
        }
    }

    Ok(())
}

async fn fetch_url(url: &str) -> Result<String, Box<dyn Error>> {
    // let user_agent = spoof_ua();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(spoof_ua())?);
    headers.insert(ACCEPT, HeaderValue::from_static(
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
    ));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static(
            "en-US,en;q=0.9,fr;q=0.8,de;q=0.7,es;q=0.6,zh;q=0.5,ja;q=0.4"
        ),
    );

    let domain = url.split('/').take(3).collect::<Vec<_>>().join("/");
    headers.insert(REFERER, HeaderValue::from_str(&domain)?);

    headers.insert(REFERER, HeaderValue::from_static("https://www.duckduckgo.com"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Cache-Control", HeaderValue::from_static("max-age=0"));

    let client = reqwest::Client::new();
    
    // let response = reqwest::get(url).await?;
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?;

    let status = response.status(); 
    if !status.is_success() {
        return Err(format!("Http error: {}", status).into());
    }

    let body = response.text().await?;
    Ok(body)
}