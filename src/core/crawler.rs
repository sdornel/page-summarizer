use std::fs::File;
use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::error::Error;
use std::borrow::Cow;

use warc::{WarcReader, WarcHeader};
use flate2::read::MultiGzDecoder;
use unicode_truncate::UnicodeTruncateStr;

pub async fn run_crawler(warc_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(warc_path)?;
    let decoder = MultiGzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let warc_reader = WarcReader::new(reader);

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

                let mut body = String::new();
                if let Err(e) = record.body().read_to_string(&mut body) {
                    eprintln!("⚠️ Skipping non-UTF8 record: {}", e);
                    continue;
                }
                
                
                let html = body
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

                // println!(
                //     "\n🔗 URL: {}\n🕒 Timestamp: {}\n🧾 Raw Text (preview):\n{}",
                //     url,
                //     timestamp,
                //     readable.unicode_truncate(1000).0
                // );

                let mut child = Command::new("/opt/venv/bin/python")
                    .arg("agents/summarize.py")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("❌ Failed to run summarize.py: {}", e))?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    write!(stdin, "{}", readable)?;
                }

                let output = child.wait_with_output()?;
                let summary = String::from_utf8_lossy(&output.stdout);

                // println!(
                //     "\n📄 URL: {}\n🕒 Timestamp: {}\n✍️ Summary:\n{}",
                //     url, timestamp, summary
                // );

                count += 1;
            }
        }
    }

    Ok(())
}