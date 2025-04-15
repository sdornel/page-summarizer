use axum::{
    routing::post,
    extract::Json,
    response::IntoResponse,
    Router,
};
use dotenv::dotenv;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener; // ✅ Required now
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
    pub model: Option<String>,
}

#[derive(Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
}

pub async fn summarize_with_proxy(payload: SummarizeRequest) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY")?;
    let model = payload.model.unwrap_or_else(|| "deepseek-chat".to_string());

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": "You are a helpful summarizer." },
            { "role": "user", "content": format!("Summarize the following:\n\n{}", payload.text) }
        ]
    });

    let client = Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let summary = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No summary returned")
        .to_string();

    Ok(summary)
}

async fn summarize(Json(payload): Json<SummarizeRequest>) -> impl IntoResponse {
    match summarize_with_proxy(payload).await {
        Ok(summary) => Json(SummarizeResponse { summary }),
        Err(e) => {
            eprintln!("❌ Proxy summarization failed: {}", e);
            Json(SummarizeResponse { summary: "(Proxy failed)".to_string() })
        }
    }
}

pub async fn start_proxy() {
    dotenv().ok();

    let app = Router::new().route("/summarize", post(summarize));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.expect("🛑 Failed to bind port");

    println!("🚀 Proxy running at http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
