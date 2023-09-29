use anyhow::{Error, Result};
use async_openai::Client;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};

pub fn get_gpt_response(window_text: String, user_speech_to_text: String) -> Result<ChatCompletionRequestMessage, Error> {

    let mut messages = messages_setup();
    messages.push(create_chat_completion_request_msg(window_text, Role::User));
    messages.push(create_chat_completion_request_msg(user_speech_to_text, Role::User));

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(120_u16)
        .messages(messages.clone())
        .build()?;

    let resp = tokio::runtime::Runtime::new().unwrap().block_on(client.chat().create(request))?;

    let resp_message = resp.choices.get(0).unwrap().message.clone();

    let bot_string = resp_message.content.as_ref().unwrap().clone();

    let new_bot_message = create_chat_completion_request_msg(
        bot_string,
        Role::Assistant);

    return Ok(new_bot_message);
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
      Ensure that your output is just the output they requested - do not ask any follow up questions or include any extra text.";
    let system_message = create_chat_completion_request_msg(system_message_content.to_string(), Role::System);

    // let user_prompt_content = get_from_store(handle, "userPrompt").unwrap_or("".to_string());
    // let user_prompt_message = create_chat_completion_request_msg("user_prompt_content".to_string(), Role::System);

    return vec![system_message]
}

