use std::fs::File;
use std::io::Read;
use anyhow::{Error, Result};
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};
use base64::encode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};
use futures_util::StreamExt;


#[derive(Serialize, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Serialize, Deserialize)]
struct MessageContent {
    content: String,
}

pub async fn get_gpt_response(app_handle: &AppHandle, messages: Vec<ChatCompletionRequestMessage>, image_path: String) -> Result<(), Error> {
    println!("Getting GPT response");
    let mut file = File::open(image_path)?;
    // Read the contents of the file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Encode the buffer as a Base64 string
    let base64_string = encode(&buffer);

    let payload = json!({
        "model": "gpt-4-vision-preview",
        "messages": [
          {
            "role": "user",
            "content": [
              {
                "type": "text",
                "text": "What’s in this image?"
              },
              {
                "type": "image_url",
                "image_url": {
                  "url": format!("data:image/jpeg;base64,{}", base64_string)
                }
              }
            ]
          }
        ],
        "stream": true,
        "max_tokens": 150,
    });

    let client = Client::new();
    let api_url = "https://api.openai.com/v1/chat/completions";

    // Make the POST request
    let response = client.post(api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", env!("OPENAI_API_KEY")))
        .json(&payload)
        .send().await?;

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?; // Get the chunk as bytes
        let chunk_str = String::from_utf8(chunk.to_vec())?;

        // Emit a Tauri event with the chunk content
        app_handle.emit_all("gpt_chunk_received", chunk_str).expect("Failed to emit event");
    }
    println!("Finished getting GPT response");
    Ok(())
}


pub fn create_chat_completion_request_msg(content: String, role: Role) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessageArgs::default()
        .content(content)
        .role(role)
        .build()
        .unwrap()
}


pub fn messages_setup() -> Vec<ChatCompletionRequestMessage> {
    let system_message_content = "This is an AI macos app where the user asks for the AI to write some text via speech-to-text, and then the text is pasted into the field that they currently have selected.\
     The user uses speech-to-text to communicate, so some of their messages may be incorrect - make assumptions based on this.\
     If the user's message is text between brackets, for example '[BLANK AUDIO]', '[phone ringing], [silence], [background noise]', then say 'I didn't catch that, please try again'.\
     The user will be unable to respond to you after you send a message, so do not ask any questions or ask for clarification.\
      Ensure that your output is just the output they requested - do not ask any follow up questions or include any extra text.\
      The next message is the OCRd text from the users active window - use it to provide context for the user.\
      The message after that is the user's prompt - respond to this";
    let system_message = create_chat_completion_request_msg(system_message_content.to_string(), Role::System);

    // let user_prompt_content = get_from_store(handle, "userPrompt").unwrap_or("".to_string());
    // let user_prompt_message = create_chat_completion_request_msg("user_prompt_content".to_string(), Role::System);

    return vec![system_message]
}

