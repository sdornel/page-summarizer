// Import from the futures crate for working with asynchronous streams.
// `stream` provides functions to create and work with streams,
// and `StreamExt` offers extension methods (like for_each_concurrent) for streams.
use futures::stream::{self, StreamExt};

// Import the reqwest Client type for making HTTP requests.
use reqwest::Client;

// Import the standard Error trait for error handling.
use std::error::Error;

// Import the standard fs module for file system operations like reading files.
use std::fs;

// Import Arc (Atomic Reference Counted pointer) for safe sharing of data across threads/tasks.
use std::sync::Arc;

// Import Mutex from tokio for asynchronous mutual exclusion when sharing data across tasks.
use tokio::sync::Mutex;

// Import the spoof_ua function from ua_generator to generate fake User-Agent strings.
use ua_generator::ua::spoof_ua;

mod generate_random_headers;
use generate_random_headers::generate_random_headers;

async fn fetch_url(url: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let headers = generate_random_headers(url)?;
    let response = client.get(url).headers(headers).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    let body = response.text().await?;
    Ok(body)
}

async fn fallback_to_puppeteer(url: &str) -> Result<String, Box<dyn Error>> {
    println!("Fallback: using Puppeteer for URL: {}", url);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let json_data = fs::read_to_string("../output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;
    println!("Starting scrape of {} URLs...", urls.len());

    let client = Client::builder().user_agent(spoof_ua()).build()?;

    // Shared failure counter wrapped in an Arc<Mutex<_>>.
    let rust_failed_count = Arc::new(Mutex::new(0));
    let concurrency_limit = 30;

    // Process URLs concurrently.
    stream::iter(urls.into_iter().enumerate())
        .for_each_concurrent(concurrency_limit, |(i, url)| {
            let client = client.clone();
            let fail_counter = rust_failed_count.clone(); // clone the Arc for this task
            async move {
                println!("Processing URL {}: {}", i, url);
                let result = match fetch_url(&url, &client).await {
                    Ok(body) => Ok(body),
                    Err(_) => fallback_to_puppeteer(&url).await,
                };
                match result {
                    Ok(body) => {
                        println!("✅ URL {}: Success ({} bytes)", i, body.len());
                        // Optionally, print a snippet:
                        // println!("Snippet: {}", &body[..std::cmp::min(200, body.len())]);
                    }
                    Err(e) => {
                        eprintln!("❌ URL {}: Failed: {}", i, e);
                        let mut count = fail_counter.lock().await;
                        *count += 1;
                    }
                }
            }
        })
        .await;

    let final_failed = *rust_failed_count.lock().await;
    println!("Total failed URLs: {}", final_failed);

    Ok(())
}