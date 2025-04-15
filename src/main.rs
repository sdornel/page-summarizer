use futures::stream::StreamExt;
use async_nats::Client as NatsConnection;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{timeout, Duration};
use ua_generator::ua::spoof_ua;
use uuid::Uuid;

mod generate_random_headers;
use generate_random_headers::generate_random_headers;
mod page_cleaner;
use page_cleaner::extract_main_content;
mod tfidf_summarizer;
use tfidf_summarizer::summarize_tfidf;
mod agents;
use agents::proxy_summarizer::{summarize_with_proxy, SummarizeRequest};

// const NATS_TIMEOUT_SECS: u64 = 15;
// // const SUMMARY_TYPES_EXPECTED: usize = 2;
// const SUMMARY_TYPES_EXPECTED: usize = 1;
const SUMMARY_SENTENCE_COUNT: usize = 5;

use async_nats::Message;
use bytes::Bytes;

async fn fetch_url(url: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let headers = generate_random_headers(url)?;
    let response = client.get(url).headers(headers).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    let body = response.text().await?;
    Ok(body)
}

// async fn publish_summarization_job(
//     nats: &NatsConnection,
//     url: &str,
//     text: &str,
//     reply_subject: &str,
//     corr_id: &str,
// ) -> Result<(), Box<dyn Error>> {
//     let payload = json!({
//         "query": url,
//         "url": url,
//         "text": text,
//         "correlation_id": corr_id,
//         "reply_to": reply_subject
//     })
//     .to_string();

//     nats.publish("summarization_job", payload.into()).await?;
//     nats.flush().await?;
//     Ok(())
// }

fn truncate_utf8(input: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(input.len());
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    &input[..end]
}

async fn fallback_to_puppeteer(
    url: &str,
    correlation_id: &str,
    reply_subject: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🧪 Triggering Puppeteer fallback for: {}", url);

    let status = tokio::process::Command::new("node")
        .arg("/app/src/scrapers-js/backup-page-opener.js")
        .arg(url)
        .arg(correlation_id)
        .arg(reply_subject)
        .status()
        .await?;

    if !status.success() {
        eprintln!("❌ Puppeteer script failed for {}", url);
    }
    Ok(())
}

// just a placeholder for now
// async fn wait_for_summarizer_ready(nats: &NatsConnection) -> Result<(), Box<dyn Error>> {
//     println!("⏳ Waiting for summarizer-agent to be ready...");

//     let mut health_sub = nats.subscribe("health.summarizer").await?;
//     // let agent_ready = timeout(Duration::from_secs(5), health_sub.next()).await;
//     let agent_ready = timeout(Duration::from_secs(100), health_sub.next()).await;

//     match agent_ready {
//         Ok(Some(_msg)) => {
//             println!("✅ Summarizer agent is ready");
//             Ok(())
//         }
//         _ => Err("Summarizer agent did not become ready in time".into()),
//     }
// }
async fn wait_for_summarizer_ready(nats: &async_nats::Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("⏳ Waiting for summarizer-agent to be ready (via request reply)...");

    // Send a request with a Bytes payload.
    let response = timeout(
        Duration::from_secs(5),
        nats.request("health.summarizer.request", Bytes::from_static(b"health?"))
    ).await??;

    // Check the reply using the 'payload' field.
    if response.payload == Bytes::from_static(b"ready") {
        println!("✅ Summarizer agent is ready (reply received)");
        Ok(())
    } else {
        Err("Summarizer agent did not respond with 'ready'".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::info!("🔧 Starting scrape...");

    let json_data = fs::read_to_string("output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;
    let urls_for_loop = urls.clone();
    println!("Starting scrape of {} URLs...", urls.len());

    let client = Client::builder().user_agent(spoof_ua()).build()?;

    let nats = async_nats::connect("nats:4222").await?;
    wait_for_summarizer_ready(&nats).await?;

    let reply_subject = format!("summarizer_response_{}", Uuid::new_v4());
    println!("🔗 NATS reply subject: {}", reply_subject);
    let mut subscriber = nats.subscribe(reply_subject.clone()).await?;

    let tx_map: Arc<Mutex<HashMap<String, mpsc::Sender<Value>>>> = Arc::new(Mutex::new(HashMap::new()));
    let summaries: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let rust_failed_count = Arc::new(Mutex::new(0));

    let tx_map_listener = tx_map.clone();
    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            if let Ok(resp_json) = serde_json::from_slice::<Value>(&msg.payload) {
                if let Some(corr_id) = resp_json.get("correlation_id").and_then(|v| v.as_str()) {
                    if let Some(sender) = tx_map_listener.lock().await.get(corr_id) {
                        let _ = sender.send(resp_json.clone()).await;
                    }
                }
            }
        }
    });

    let concurrency_limit = 30;
    let reply_subject_cloned = reply_subject.clone();
    futures::stream::iter(urls_for_loop.into_iter().enumerate())
        .for_each_concurrent(concurrency_limit, |(i, url)| {
            let client = client.clone();
            let summaries = summaries.clone();
            let fail_counter = rust_failed_count.clone();
            let reply_subject = reply_subject_cloned.clone();

            async move {
                println!("🌐 [{}] Processing: {}", i, url);
                let corr_id = Uuid::new_v4().to_string();
                let body_result = match fetch_url(&url, &client).await {
                    Ok(body) => Ok(body),
                    Err(e) => {
                        eprintln!("❌ URL {}: Fetch failed: {}", i, e);
                        if let Err(e) = fallback_to_puppeteer(&url, &corr_id, &reply_subject).await {
                            eprintln!("⚠️ Puppeteer fallback failed: {}", e);
                        }
                        Err(e)
                    }
                };

                match body_result {
                    Ok(body) => {
                        let clean_text = extract_main_content(&body);
                        let trimmed_text = truncate_utf8(&clean_text, 1_000_000);

                        let tfidf_sentences = summarize_tfidf(trimmed_text, SUMMARY_SENTENCE_COUNT);
                        let tfidf_summary = tfidf_sentences.join(". ") + ".";

                        let proxy_summary = match summarize_with_proxy(SummarizeRequest {
                            text: trimmed_text.to_string(),
                            model: Some("deepseek-chat".to_string())
                        }).await {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("⚠️ Proxy summarization failed: {}", e);
                                "(Proxy failed)".to_string()
                            }
                        };

                        let summary_json = json!({
                            "correlation_id": corr_id,
                            "summary_type": "combined",
                            "tfidf_summary": tfidf_summary,
                            "proxy_summary": proxy_summary
                        });

                        let key = format!("{}:combined", url);
                        summaries.lock().await.insert(key, summary_json);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to process URL {}: {}", url, e);
                        *fail_counter.lock().await += 1;
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

    fs::create_dir_all("output")?;
    fs::write("output/summaries.json", serde_json::to_string_pretty(&summary_result)?)?;
    println!("Saved summaries to output/summaries.json");

    Ok(())
}
