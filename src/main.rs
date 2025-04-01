use ua_generator::ua::spoof_ua; // fake user agent
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER}; // allows to set headers
use std::fs; // file parsing
use std::error::Error; // allows us to use Result for error handling

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let json_data = fs::read_to_string("../output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;

    println!("Raw JSON data:\n{}", json_data);
    let client = reqwest::Client::builder()
    .user_agent(spoof_ua()) // backup user-agent
    .build()?;

    let mut rust_failed_count: i32 = 0;
    for (i, url) in urls.iter().enumerate() {
        // i need to spin up lots of threads to make it all faster
        println!("{} => {}", i, url);
        let result = async {
            match fetch_url(url, &client).await {
                Ok(body) => Ok(body),
                Err(_) => fallback_to_puppeteer(url).await,
            }
        }.await;

        match result {
            Ok(body) => println!("✅ Final Success: {} bytes\n", body.len()),
            Err(error) => {
                println!("❌ Failed to fetch: {}\n", error);
                rust_failed_count += 1;
            },        
        }
    }
    println!("rust_failed_count: {}", rust_failed_count); // measure how effective my fetch_url function is
    Ok(())
}

async fn fetch_url(url: &str, client: &reqwest::Client,) -> Result<String, Box<dyn Error>> {
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

    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Cache-Control", HeaderValue::from_static("max-age=0"));
    
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

async fn fallback_to_puppeteer(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Failed to retrieve page. Falling back to puppeteer!");
    let output = tokio::process::Command::new("node")
        .arg("scrapers-js/backup-page-opener.js")
        .arg(url)
        .output()
        .await?;

    if !output.status.success() {
        return Err(format!("Puppeteer failed: {:?}", output).into());
    }

    String::from_utf8(output.stdout).map_err(Into::into)
}