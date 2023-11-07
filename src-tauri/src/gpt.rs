use std::env;
use anyhow::{Error, Result};
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};
use base64::encode;
use reqwest::{Client, header};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use futures_util::StreamExt;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


#[derive(Debug)]
pub struct GptClient {
    app_handle: AppHandle,
    client: Client,
    api_url: String,
}

impl GptClient {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            client: Client::new(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }

    pub async fn get_gpt_response(&self, mut messages: Vec<ChatCompletionRequestMessage>, image_path: String) -> Result<()> {
        if self.is_testing_env() {
            self.use_mocked_response().await?;
            return Ok(());
        }

        let base64_string = self.encode_image(image_path).await?;
        let json_messages = self.prepare_messages_for_payload(messages, &base64_string);
        let payload = self.build_payload(json_messages)?;
        self.send_request_and_emit_events(payload).await?;
        Ok(())
    }

    async fn encode_image(&self, image_path: String) -> Result<String> {
        let buffer = fs::read(&image_path).await?;
        Ok(encode(&buffer))
    }

    fn prepare_messages_for_payload(&self, messages: Vec<ChatCompletionRequestMessage>, base64_string: &str) -> Vec<Value> {
        messages.into_iter().map(|msg| {
            let content = match msg.role {
                Role::User => json!([
                    {
                        "type": "text",
                        "text": msg.content.unwrap_or_default(),
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpeg;base64,{}", base64_string)
                        }
                    }
                ]),
                Role::Assistant | Role::System => json!([{
                    "type": "text",
                    "text": msg.content.unwrap_or_default(),
                }]),
                _ => json!({
                    "error": "Role not specified"
                })
            };

            json!({
                "role": msg.role.to_string(),
                "content": content,
            })
        }).collect()
    }

    fn build_payload(&self, json_messages: Vec<Value>) -> Result<Value> {
        Ok(json!({
            "model": "gpt-4-vision-preview",
            "messages": json_messages,
            "stream": true,
            "max_tokens": 150,
        }))
    }

    async fn send_request_and_emit_events(&self, payload: Value) -> Result<()> {
        let openai_api_key = env::var("OPENAI_API_KEY")?;
        let response = self.client
            .post(&self.api_url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", openai_api_key))
            .json(&payload)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut json_parser = JSONBufferParser::new();

        self.app_handle.emit_all("gpt_stream_start", json!({ "status": "start" }))?;
        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = String::from_utf8(chunk.to_vec())?;

            // Append the chunk to the JSON buffer and process any complete JSON objects
            json_parser.append(&chunk_str);
            for content in json_parser.extract_content() {
                self.app_handle.emit_all("gpt_chunk_received", content)?;
            }
        }
        Ok(())
    }

    async fn use_mocked_response(&self) -> Result<()> {
        let responses = self.read_mocked_responses("openai_response.txt").await?;
        for response in responses {
            self.app_handle.emit_all("gpt_chunk_received", response)?;
        }
        Ok(())
    }

    async fn read_mocked_responses(&self, file_path: &str) -> Result<Vec<String>> {
        let buffer = fs::read(file_path).await?;
        Ok(buffer
            .split_inclusive(|&x| x == b'\n')
            .filter_map(|bytes| String::from_utf8(bytes.to_vec()).ok())
            .collect())
    }

    fn is_testing_env(&self) -> bool {
        let is_testing_env = env::var("TESTING_ENV").map(|val| val == "true").unwrap_or(false);
        println!("is_testing_env: {}", is_testing_env);
        is_testing_env
    }
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
     The user will be unable to respond to you after you send a message, so do not ask any questions or ask for clarification.\
      Ensure that your output is just the output they requested - do not ask any follow up questions or include any extra text.";
    let system_message = create_chat_completion_request_msg(system_message_content.to_string(), Role::System);

    // let user_prompt_content = get_from_store(handle, "userPrompt").unwrap_or("".to_string());
    // let user_prompt_message = create_chat_completion_request_msg("user_prompt_content".to_string(), Role::System);

    return vec![system_message]
}


// JSONBufferParser helps in buffering chunks and extracting complete JSON objects.
struct JSONBufferParser {
    buffer: String,
}

impl JSONBufferParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn append(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
    }

    fn extract_content(&mut self) -> Vec<String> {
        let mut contents = Vec::new();
        loop {
            match self.find_json_object_boundaries() {
                Some((start, end)) => {
                    let json_str = self.buffer[start..=end].to_owned();
                    // Attempt to parse the JSON object and extract the content.
                    if let Some(content) = extract_content_from_json(&json_str) {
                        contents.push(content);
                    }
                    self.buffer.drain(..=end);
                }
                None => break,
            }
        }
        contents
    }

    fn find_json_object_boundaries(&self) -> Option<(usize, usize)> {
        let mut depth = 0;
        let mut start_index = None;
        for (i, ch) in self.buffer.char_indices() {
            match ch {
                '{' => {
                    if depth == 0 {
                        start_index = Some(i);
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 && start_index.is_some() {
                        return Some((start_index.unwrap(), i));
                    }
                }
                _ => {}
            }
        }
        None
    }
}

fn extract_content_from_json(json_str: &str) -> Option<String> {
    let value: serde_json::Result<serde_json::Value> = serde_json::from_str(json_str);

    if let Ok(val) = value {
        if let Some(content) = val["choices"].get(0)
            .and_then(|choice| choice["delta"].get("content"))
            .and_then(|content| content.as_str()) {
            return Some(content.to_owned());
        }
    }

    None
}