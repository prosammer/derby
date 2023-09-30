use std::collections::HashMap;
use std::env;
use anyhow::Result;

use bytes::Bytes;
use reqwest::Error;

pub async fn text_to_speech(voice_id: &str, text: String) -> Result<Bytes, Error> {
    let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}/stream", voice_id);
    let mut data = HashMap::new();
    data.insert("text", text);
    let api_key = env::var("ELEVEN_LABS_API_KEY").expect("ELEVEN_LABS_API_KEY must be set");
    let client = reqwest::Client::new();
    let res = client.post(&url)
        .header("Accept", "audio/mpeg")
        .header("Content-Type", "application/json")
        .header("xi-api-key", api_key)
        .json(&data)
        .send()
        .await?;

    return Ok(res.bytes().await?);
}