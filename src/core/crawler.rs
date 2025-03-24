use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::error::Error;
use std::borrow::Cow;

use warc::{WarcReader, WarcHeader};
use flate2::read::MultiGzDecoder;
use unicode_truncate::UnicodeTruncateStr;
use serde_json::json;

fn summarize_with_python(content: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut child = Command::new("/opt/venv/bin/python")
        .arg("agents/summarize.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content)?;
    }

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn is_relevant_with_python(topic: &str, content: &str) -> Result<bool, Box<dyn Error>> {
    let input = json!({ "topic": topic, "content": content }).to_string();

    let mut child = Command::new("/opt/venv/bin/python")
        .arg("agents/relevance.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    Ok(result.trim() == "RELEVANT")
}

pub async fn run_crawler(warc_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(warc_path)?;
    let decoder = MultiGzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let warc_reader = WarcReader::new(reader);
    let topic = "Ethics and AI";

    let mut count = 0;
    // let max_pages = 1000;

    for result in warc_reader.iter_records() {
        // if count >= max_pages {
        //     break;
        // }

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("⚠️ Skipping record due to error: {}", e);
                continue;
            }
        };

        if let Some(header) = record.header(WarcHeader::WarcType) {
            if header == "response" {
                let url = record
                    .header(WarcHeader::TargetURI)
                    .unwrap_or(Cow::Borrowed("unknown"));

                let timestamp = record
                    .header(WarcHeader::Date)
                    .unwrap_or(Cow::Borrowed("unknown"));

                let mut raw_body = Vec::new();
                if let Err(e) = record.body().read_to_end(&mut raw_body) {
                    eprintln!("⚠️ Failed to read record body: {}", e);
                    continue;
                }

                let is_relevant = is_relevant_with_python(topic, &String::from_utf8_lossy(&raw_body))?;
                if !is_relevant {
                    continue;
                }

                let body_str = match String::from_utf8(raw_body.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        let summary = summarize_with_python(&raw_body)?;
                        println!(
                            "\n📄 URL: {}\n🕒 Timestamp: {}\n✍️ Summary:\n{}",
                            url, timestamp, summary
                        );
                        count += 1;
                        continue;
                    }
                };

                let html = body_str
                    .splitn(2, "\r\n\r\n")
                    .nth(1)
                    .unwrap_or("")
                    .trim();

                if !html.contains("<html") {
                    continue;
                }

                let readable = html2text::from_read(html.as_bytes(), 80);
                let lower = readable.to_lowercase();
                if lower.contains("accept cookies") || lower.contains("404 not found") {
                    continue;
                }

                if !is_relevant_with_python(topic, &readable)? {
                    continue;
                }

                let summary = summarize_with_python(html.as_bytes())?;

                println!(
                    "\n📄 URL: {}\n🕒 Timestamp: {}\n✍️ Summary:\n{}",
                    url, timestamp, summary
                );

                count += 1;
            }
        }
    }

    Ok(())
}