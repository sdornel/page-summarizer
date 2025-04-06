use futures::stream::StreamExt;
use async_nats::Client as NatsConnection;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use ua_generator::ua::spoof_ua;
use uuid::Uuid;

mod generate_random_headers;
use generate_random_headers::generate_random_headers;
mod page_cleaner;
use page_cleaner::extract_main_content;

async fn fetch_url(url: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let headers = generate_random_headers(url)?;
    let response = client.get(url).headers(headers).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    let body = response.text().await?;
    Ok(body)
}

async fn publish_summarization_job(
    nats: &NatsConnection,
    url: &str,
    text: &str,
    reply_subject: &str,
    corr_id: &str,
) -> Result<(), Box<dyn Error>> {
    let payload = json!({
        "query": url,
        "url": url,
        "text": text,
        "correlation_id": corr_id,
        "reply_to": reply_subject
    })
    .to_string();

    nats.publish("summarization_job", payload.into()).await?;
    Ok(())
}

// async fn fallback_to_puppeteer(url: &str) -> Result<String, Box<dyn Error>> {
//     println!("Fallback: using Puppeteer for URL: {}", url);
//     let output = tokio::process::Command::new("node")
//         .arg("/app/src/scrapers-js/backup-page-opener.js")
//         .arg(url)
//         .output()
//         .await?;
//     if !output.status.success() {
//         eprintln!("❌ Puppeteer script stderr: {}", String::from_utf8_lossy(&output.stderr));
//         return Err(format!("Puppeteer failed: {:?}", output).into());
//     }
//     String::from_utf8(output.stdout).map_err(Into::into)
// }

async fn fallback_to_puppeteer(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Fallback: using Puppeteer for URL: {}", url);

    let result = timeout(
        Duration::from_secs(15), // 👈 timeout after 15 seconds
        tokio::process::Command::new("node")
            .arg("/app/src/scrapers-js/backup-page-opener.js")
            .arg(url)
            .output(),
    )
    .await;

    match result {
        Ok(output_result) => {
            let output = output_result?;
            if !output.status.success() {
                eprintln!("❌ Puppeteer stderr: {}", String::from_utf8_lossy(&output.stderr));
                return Err(format!("Puppeteer failed: {:?}", output).into());
            } else {
                println!("✅ Puppeteer stdout: {}", String::from_utf8_lossy(&output.stdout));
            }
            Ok(String::from_utf8(output.stdout)?)
        }
        Err(_) => {
            Err("❌ Puppeteer timed out".into())
        }
    }
}

fn truncate_utf8(input: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(input.len());
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    &input[..end]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::info!("🔧 Starting scrape...");
    log::debug!("This is debug info");
    log::warn!("Something sketchy happened");
    log::error!("Something failed");

    let json_data = fs::read_to_string("output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;
    let urls_for_loop = urls.clone();
    println!("Starting scrape of {} URLs...", urls.len());

    let client = Client::builder().user_agent(spoof_ua()).build()?;

    let nats = async_nats::connect("nats:4222").await?;
    let reply_subject = format!("summarizer_response_{}", Uuid::new_v4());
    let sub = Arc::new(Mutex::new(nats.subscribe(reply_subject.clone()).await?));

    let summaries: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let rust_failed_count = Arc::new(Mutex::new(0));

    let concurrency_limit = 30;
    futures::stream::iter(urls_for_loop.into_iter().enumerate())
        .for_each_concurrent(concurrency_limit, |(i, url)| {
            let client = client.clone();
            let nats = nats.clone();
            let reply_subject = reply_subject.clone();
            let summaries = summaries.clone();
            let fail_counter = rust_failed_count.clone();
            let sub = sub.clone();

            async move {
                println!("Processing URL {}: {}", i, url);
                let body_result = match fetch_url(&url, &client).await {
                    Ok(body) => Ok(body),
                    Err(_) => fallback_to_puppeteer(&url).await,
                };

                match body_result {
                    Ok(body) => {
                        let clean_text = extract_main_content(&body);
                        let trimmed_text = truncate_utf8(&clean_text, 1_000_000);
                        let corr_id = Uuid::new_v4().to_string();

                        // if let Err(e) = publish_summarization_job(&nats, &url, &clean_text, &reply_subject, &corr_id).await {
                        if let Err(e) = publish_summarization_job(&nats, &url, &trimmed_text, &reply_subject, &corr_id).await {
                            eprintln!("❌ Failed to publish job for {}: {}", url, e);
                            let mut count = fail_counter.lock().await;
                            *count += 1;
                        } else {
                            let mut seen_types = HashSet::new();
                            while seen_types.len() < 2 {
                                let mut sub_lock = sub.lock().await;
                                if let Some(msg) = sub_lock.next().await {
                                    if let Ok(resp_json) = serde_json::from_slice::<Value>(&msg.payload) {
                                        if let Some(resp_corr_id) = resp_json.get("correlation_id").and_then(|v| v.as_str()) {
                                            if resp_corr_id == corr_id {
                                                if let Some(summary_type) = resp_json.get("summary_type").and_then(|v| v.as_str()) {
                                                    let key = format!("{}:{}", url, summary_type);
                                                    let clone = resp_json.clone();
                                                    summaries.lock().await.insert(key, clone);

                                                    seen_types.insert(summary_type.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
    let sum_map = summaries.lock().await;
    let summary_result = json!({
        "meta": {
            "total_urls": urls.len(),
            "successful": urls.len() - final_failed,
            "failed": final_failed
        },
        "summaries": *sum_map
    });
    println!("SUMMARIES: {}", summary_result);
    fs::create_dir_all("output")?;
    fs::write("output/summaries.json", serde_json::to_string_pretty(&summary_result)?)?;
    println!("Saved summaries to output/summaries.json");

    Ok(())
}
