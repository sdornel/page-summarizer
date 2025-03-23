mod core;

use core::crawler::run_crawler;

#[tokio::main]
async fn main() {
    // let warc_path = "./example.warc.gz"; // placeholder
    let warc_path = "/home/kai/Downloads/CC-MAIN-20240318101932-20240318131932-00000.warc.gz";

    match run_crawler(warc_path).await {
        Ok(_) => println!("✅ Crawling completed."),
        Err(e) => eprintln!("❌ Crawling failed: {}", e),
    }

}


// mod core;

// use core::{common_crawl::run_crawler, live_spider_crawl::run_live_crawler};

// #[tokio::main]
// async fn main() {
//     let args: Vec<String> = std::env::args().collect();

//     if args.len() < 3 {
//         eprintln!("Usage: cargo run -- <crawl|live> <path_or_url>");
//         return;
//     }

//     let mode = &args[1];
//     let input = &args[2];

//     match mode.as_str() {
//         "crawl" => match run_crawler(input).await { // use commoncrawl
//             Ok(_) => println!("✅ WARC crawling completed."),
//             Err(e) => eprintln!("❌ Crawling failed: {}", e),
//         },
//         // "live" => match run_live_crawler(input).await { // live crawling
//         //     Ok(_) => println!("✅ Live crawling completed."),
//         //     Err(e) => eprintln!("❌ Live crawling failed: {}", e),
//         // },
//         _ => eprintln!("Unknown mode: {}. Use 'crawl' or 'live'", mode),
//     }
// }