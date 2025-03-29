
use std::fs; // file parsing
use std::error::Error; // allows us to use Result for error handling
use ua_generator::ua::spoof_ua; // fake user agent
use reqwest::header::USER_AGENT; // allows to set user agent

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let json_data = fs::read_to_string("../output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;

    println!("Raw JSON data:\n{}", json_data);

    for (i, url) in urls.iter().enumerate() {
        println!("{} => {}", i, url);
        match fetch_url(url).await {
            Ok(body) => println!("✅ Success: {} bytes\n", body.len()),
            Err(error) => println!("❌ Failed to fetch: {}\n", error),
        }
    }

    Ok(())
}

async fn fetch_url(url: &str) -> Result<String, Box<dyn Error>> {
    let user_agent = spoof_ua();

    let client = reqwest::Client::new();
    
    // let response = reqwest::get(url).await?;
    let response = client
        .get(url)
        .header(USER_AGENT, user_agent)
        .send()
        .await?;

    let status = response.status(); 
    if !status.is_success() {
        return Err(format!("Http error: {}", status).into());
    }

    let body = response.text().await?;
    Ok(body)
}