
use std::fs; // file parsing
use std::error::Error; // allows us to use Result for error handling

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let json_data = fs::read_to_string("../output/urls.json")?;
    let urls: Vec<String> = serde_json::from_str(&json_data)?;

    println!("Raw JSON data:\n{}", json_data);

    // for url in &urls {
    //     println!("{}", url);
    // }

    Ok(())
}
