use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};
use warc::WarcReader;
use html2text::from_read;
use std::error::Error;

pub async fn run_crawler(warc_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(warc_path)?;
    let reader = BufReader::new(file);
    let warc_reader = WarcReader::new(reader);

    for result in warc_reader.iter_records() {
        let record = result?;

        if let Some(header) = record.header("WARC-Type") {
            if header == "response" {
                let url = record.header("WARC-Target-URI").unwrap_or("unknown");
                let timestamp = record.header("WARC-Date").unwrap_or("unknown");

                // Convert HTML to plain text
                let html = from_read(record.body(), 80);

                // Spawn Python summarizer
                let mut child = Command::new("python3")
                    .arg("agents/summarize.py")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    write!(stdin, "{}", html)?;
                }

                let output = child.wait_with_output()?;
                let summary = String::from_utf8_lossy(&output.stdout);

                println!(
                    "\n📄 URL: {}\n🕒 Timestamp: {}\n✍️ Summary:\n{}",
                    url, timestamp, summary
                );
            }
        }
    }

    Ok(())
}