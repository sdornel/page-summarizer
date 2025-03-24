mod core;

use core::crawler::run_crawler;

#[tokio::main]
async fn main() {
    let warc_path = "./CC-MAIN-20250206114225-20250206144225-00000.warc.gz";

    match run_crawler(warc_path).await {
        Ok(_) => println!("✅ Crawling completed."),
        Err(e) => eprintln!("❌ Crawling failed: {}", e),
    }
}
