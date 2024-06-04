use std::time::Duration;

use super::helper::*;
use anyhow::{anyhow, Error as AnyError};
use axum::{
    extract::{Json, Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse, Response,
    },
};
use dify_client::{
    api::Api,
    http::{header, Request as HttpRequest},
    request::ChatMessagesRequest,
    response::{ErrorResponse, SseMessageEvent},
};
use futures::stream;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio_stream::StreamExt;

/// Checks if the request method is OPTIONS.
/// If the request method is OPTIONS, it returns a 204 No Content response.
/// If the request method is not OPTIONS, it calls the next middleware.
/// This function is called before the chat completions handler.
/// It is used to handle preflight requests.
pub async fn check_method(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.method() == Method::OPTIONS {
        Err(StatusCode::NO_CONTENT)
    } else {
        Ok(next.run(req).await)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ChatCompletionRequest {
    /// The messages
    /// A list of messages comprising the conversation so far.
    /// Example: [
    ///     {"role": "user", "content": "Hello, how are you?"},
    ///     {"role": "system", "content": "I'm doing well, thank you."},
    ///     {"role": "user", "content": "That's great to hear. How can I help you today?"}
    /// ]
    messages: Vec<Message>,
    /// ID of the model to use. See the model endpoint compatibility table for details on which models work with the Chat API.
    /// Example: "gpt-3.5-turbo"
    model: String,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    /// Default: 0.0
    frequency_penalty: Option<f64>,
    /// Modify the likelihood of specified tokens appearing in the completion.
    logit_bias: Option<JsonValue>,
    /// Whether to return log probabilities of the output tokens or not. If true, returns the log probabilities of each output token returned in the content of message.
    /// Default: false
    logprobs: Option<bool>,
    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position, each with an associated log probability. logprobs must be set to true if this parameter is used.
    /// Default: 0
    top_logprobs: Option<u64>,
    /// The maximum number of tokens that can be generated in the chat completion.
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    max_tokens: Option<u64>,
    /// How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep n as 1 to minimize costs.
    /// Default: 1
    n: Option<u64>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    /// Default: 0.0
    presence_penalty: Option<f64>,
    /// An object specifying the format that the model must output. Compatible with GPT-4 Turbo and all GPT-3.5 Turbo models newer than gpt-3.5-turbo-1106.
    /// Setting to { "type": "json_object" } enables JSON mode, which guarantees the message the model generates is valid JSON.
    /// Important: when using JSON mode, you must also instruct the model to produce JSON yourself via a system or user message. Without this, the model may generate an unending stream of whitespace until the generation reaches the token limit, resulting in a long-running and seemingly "stuck" request. Also note that the message content may be partially cut off if finish_reason="length", which indicates the generation exceeded max_tokens or the conversation exceeded the max context length.
    /// Example: {"type": "json_object"}
    response_format: Option<ResponseFormat>,
    /// If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same seed and parameters should return the same result. Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    /// Default: null
    seed: Option<u64>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    stop: Option<JsonValue>,
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only server-sent events as they become available, with the stream terminated by a data: [DONE] message.
    /// Default: false
    stream: Option<bool>,
    /// Options for streaming response. Only set this when you set stream: true.
    /// Default: null
    stream_options: Option<StreamOptions>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    /// Default: 1.0
    temperature: Option<f64>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    /// Default: 1.0
    top_p: Option<f64>,
    /// A list of tools the model may call. Currently, only functions are supported as a tool. Use this to provide a list of functions the model may generate JSON inputs for. A max of 128 functions are supported.
    tools: Option<JsonValue>,
    /// Controls which (if any) tool is called by the model. none means the model will not call any tool and instead generates a message. auto means the model can pick between generating a message or calling one or more tools. required means the model must call one or more tools. Specifying a particular tool via {"type": "function", "function": {"name": "my_function"}} forces the model to call that tool.
    /// none is the default when no tools are present. auto is the default if tools are present.
    tool_choice: Option<JsonValue>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. Learn more.
    /// Example: "apiuser"
    user: Option<String>,
    /// Deprecated in favor of tool_choice.
    /// Controls which (if any) function is called by the model. none means the model will not call a function and instead generates a message. auto means the model can pick between generating a message or calling a function. Specifying a particular function via {"name": "my_function"} forces the model to call that function.
    function_call: Option<JsonValue>,
    /// Deprecated in favor of tools.
    /// A list of functions the model may generate JSON inputs for.
    functions: Option<JsonValue>,
}

/// An object specifying the format that the model must output.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    type_: ResponseFormatType,
}

/// The format that the model must output.
/// Setting to { "type": "json_object" } enables JSON mode, which guarantees the message the model generates is valid JSON.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    JsonObject,
    Text,
}

/// Options for streaming response.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct StreamOptions {
    include_usage: Option<bool>,
}

/// A message in the conversation.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    /// The role of the message.
    role: Role,
    /// The content of the message.
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Default, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    System,
    #[default]
    Assistant,
    Tool,
    Function,
}

/// The chat completion struct
/// Represents a chat completion response returned by model, based on the provided input.
#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionResponse {
    /// A unique identifier for the chat completion.
    id: String,
    /// A list of chat completion choices. Can be more than one if n is greater than 1.
    choices: Vec<ChatCompletionChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    created: u64,
    /// The model used for the chat completion.
    model: String,
    /// This fingerprint represents the backend configuration that the model runs with.
    /// Can be used in conjunction with the seed request parameter to understand when backend changes have been made that might impact determinism.
    system_fingerprint: String,
    /// The object type, which is always chat.completion.
    object: ObjectKind,
    /// Usage statistics for the completion request.
    usage: Usage,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum ObjectKind {
    #[default]
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "chat.completion.chunk")]
    ChatCompletionChunk,
}

/// A chat completion choice.
/// Represents a chat completion choice returned by the model.
#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionChoice {
    /// The index of the choice in the list of choices.
    index: u64,
    /// The reason the model stopped generating tokens.
    finish_reason: FinishReason,
    /// The log probability of the choice.
    logprobs: Option<JsonValue>,
    /// The text of the choice.
    message: Message,
}

/// The reason the model stopped generating tokens.
#[allow(dead_code)]
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// If the model hit a natural stop point or a provided stop sequence.
    #[default]
    Stop,
    /// If the maximum number of tokens specified in the request was reached.
    Length,
    /// If content was omitted due to a flag from our content filters.
    ContentFilter,
    /// If the model called a tool.
    ToolCalls,
    /// (deprecated) If the model called a function.
    FunctionCall,
}

/// Usage statistics for the completion request.
/// Represents the usage statistics for a chat completion request.
#[derive(Serialize, Debug, Default)]
pub struct Usage {
    /// Number of tokens in the generated completion.
    completion_tokens: u64,
    /// Number of tokens in the prompt.
    prompt_tokens: u64,
    /// Total number of tokens used in the request (prompt + completion).
    total_tokens: u64,
}

// The chat completion chunk object
// Represents a streamed chunk of a chat completion response returned by model, based on the provided input.
#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionChunkResponse {
    /// A unique identifier for the chat completion. Each chunk has the same ID.
    id: String,
    /// A list of chat completion choices. Can contain more than one elements if n is greater than 1. Can also be empty for the last chunk if you set stream_options: {"include_usage": true}.
    choices: Vec<ChatCompletionChunkChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created. Each chunk has the same timestamp.
    created: u64,
    /// The model to generate the completion.
    model: String,
    /// This fingerprint represents the backend configuration that the model runs with. Can be used in conjunction with the seed request parameter to understand when backend changes have been made that might impact determinism.
    system_fingerprint: String,
    /// The object type, which is always chat.completion.chunk.
    object: ObjectKind,
    /// An optional field that will only be present when you set stream_options: {"include_usage": true} in your request. When present, it contains a null value except for the last chunk which contains the token usage statistics for the entire request.
    usage: Option<Usage>,
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionChunkChoice {
    /// The index of the choice in the list of choices.
    index: u64,
    /// The reason the model stopped generating tokens.
    finish_reason: Option<FinishReason>,
    /// The log probability of the choice.
    logprobs: Option<JsonValue>,
    /// A chat completion delta generated by streamed model responses.
    delta: JsonValue,
}

/// Extracts the Bearer token from the Authorization header.
fn get_bearer_token(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers.get(header::AUTHORIZATION);
    let token = auth_header
        .ok_or(anyhow!("Authorization header not found"))?
        .to_str()
        .map_err(|_| anyhow!("Authorization header is not a valid string"))?
        .split(' ')
        .nth(1)
        .ok_or(anyhow!("Bearer Token not found"))?;
    Ok(token.to_owned())
}

/// Sets the Authorization header with a Bearer token.
fn set_bearer_auth(mut req: HttpRequest, token: &str) -> HttpRequest {
    let mut bearer_auth = header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
    bearer_auth.set_sensitive(true);
    req.headers_mut().insert(header::AUTHORIZATION, bearer_auth);
    req
}

/// Parses a JSON value as a u64.
/// If the value is not a number, it returns 0.
fn parse_as_u64(value: Option<&JsonValue>) -> u64 {
    value.and_then(|v| v.as_u64()).unwrap_or_default()
}

/// Handles the chat completions request.
/// This function is called when the client sends a POST request to /chat_completions.
pub async fn chat_completions_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Result<Response, AppError> {
    // Constructs a query string that includes the talk history and a question.
    let messages = payload.messages;
    let last_message = messages.last().ok_or(anyhow!("No messages provided"))?;
    let query_string = format!(
        "here is our talk history:\n'''\n{}\n'''\n\nhere is my question:\n{}",
        messages
            .iter()
            .take(messages.len() - 1)
            .map(|message| format!("{}: {}", message.role, message.content))
            .collect::<Vec<String>>()
            .join("\n"),
        last_message.content
    );

    let req_data = ChatMessagesRequest {
        query: query_string,
        user: payload.user.unwrap_or("unknow_user".into()),
        auto_generate_name: false,
        ..Default::default()
    };
    let mut api = state.dify.api();
    let token = get_bearer_token(&headers).ok();
    if let Some(token) = token {
        log::debug!("User Custom Token: {}", token);
        api.before_send(move |req| set_bearer_auth(req, token.as_str()));
    }

    if payload.stream.is_none() || !payload.stream.unwrap() {
        // Blocking chat completions
        chat_completions(&api, req_data.clone(), payload.model.as_str()).await
    } else {
        // Stream the chat completions
        chat_completions_stream(&api, req_data.clone(), payload.model.as_str()).await
    }
}

/// Handles the chat completions request.
/// It uses the `Api` instance from the `AppState` to send a request to the OpenAI API.
/// It returns a response with the chat completions.
async fn chat_completions<'a>(
    api: &Api<'a>,
    req_data: ChatMessagesRequest,
    model: &'a str,
) -> Result<Response, AppError> {
    log::debug!("Chat Completions Block Request: {:?}", req_data);
    let system_fingerprint = String::from("fp_44709d6fcb");
    let resp = api.chat_messages(req_data).await.map_err(|e| {
        e.downcast::<ErrorResponse>()
            .and_then(|err_resp| Err(anyhow!(err_resp.message)))
            .unwrap_or_else(|e| anyhow!(e))
    })?;
    let usage = resp.metadata.get("usage").unwrap_or(&JsonValue::Null);
    let response = ChatCompletionResponse {
        id: resp.base.message_id,
        choices: vec![ChatCompletionChoice {
            message: Message {
                role: Role::Assistant,
                content: resp.answer,
            },
            ..Default::default()
        }],
        created: resp.base.created_at,
        model: model.to_owned(),
        system_fingerprint,
        object: ObjectKind::ChatCompletion,
        usage: Usage {
            completion_tokens: parse_as_u64(usage.get("completion_tokens")),
            prompt_tokens: parse_as_u64(usage.get("prompt_tokens")),
            total_tokens: parse_as_u64(usage.get("total_tokens")),
        },
    };
    return Ok(serde_json::to_string(&response)?.into_response());
}

/// Handles the chat completions stream request.
/// It streams the chat completions to the client.
/// The client can use the stream to display the chat completions in real-time.
async fn chat_completions_stream<'a>(
    api: &Api<'a>,
    req_data: ChatMessagesRequest,
    model: &'a str,
) -> Result<Response, AppError> {
    log::debug!("Chat Completions Streaming Request: {:?}", req_data);
    let system_fingerprint = String::from("fp_44709d6fcb");
    let stream = api.chat_messages_stream(req_data).await?;
    let model = model.to_owned();

    let alive_duration = Duration::from_secs(30);
    let stream_default = stream::iter([SseEvent::default()
        .comment("streaming chat completions")
        .retry(alive_duration)]);
    let stream_msg = stream.map(move |result| match result {
        Ok(event) => match event {
            SseMessageEvent::Message {
                answer, id, base, ..
            }
            | SseMessageEvent::AgentMessage {
                answer, id, base, ..
            } => {
                let base_ref = base.as_ref();
                let message_id = base_ref.map(|b| b.message_id.clone()).unwrap_or(id);
                let created_at = base_ref.map(|b| b.created_at.clone()).unwrap_or(0);
                let response = ChatCompletionChunkResponse {
                    id: message_id,
                    choices: vec![ChatCompletionChunkChoice {
                        delta: serde_json::json!(Message {
                            role: Role::Assistant,
                            content: answer,
                        }),
                        ..Default::default()
                    }],
                    created: created_at,
                    model: model.clone(),
                    system_fingerprint: system_fingerprint.clone(),
                    object: ObjectKind::ChatCompletionChunk,
                    usage: None,
                };
                SseEvent::default().json_data(response).unwrap()
            }
            SseMessageEvent::MessageEnd {
                id, base, metadata, ..
            } => {
                let base_ref = base.as_ref();
                let message_id = base_ref.map(|b| b.message_id.clone()).unwrap_or(id);
                let created_at = base_ref.map(|b| b.created_at.clone()).unwrap_or(0);
                let usage = metadata.get("usage").unwrap_or(&JsonValue::Null);
                let response = ChatCompletionChunkResponse {
                    id: message_id,
                    choices: vec![ChatCompletionChunkChoice {
                        delta: serde_json::json!({}),
                        finish_reason: Some(FinishReason::Stop),
                        ..Default::default()
                    }],
                    created: created_at,
                    model: model.clone(),
                    system_fingerprint: system_fingerprint.clone(),
                    object: ObjectKind::ChatCompletionChunk,
                    usage: Some(Usage {
                        completion_tokens: parse_as_u64(usage.get("completion_tokens")),
                        prompt_tokens: parse_as_u64(usage.get("prompt_tokens")),
                        total_tokens: parse_as_u64(usage.get("total_tokens")),
                    }),
                };
                SseEvent::default().json_data(response).unwrap()
            }
            SseMessageEvent::Error { message, .. } => {
                let message = format!("upstream: {message}");
                let err = serde_json::json!({ "error": {"message":message} });
                SseEvent::default().json_data(err).unwrap()
            }
            _ => {
                let event_name = serde_json::json!(event)
                    .get("event")
                    .map(|v| v.to_string())
                    .unwrap_or("unknown".into());
                let comment = format!("skip dify message event: {}", event_name.trim_matches('"'));
                SseEvent::default().comment(comment)
            }
        },
        Err(e) => {
            let message = format!("upstream: {}", e.to_string());
            let err = serde_json::json!({ "error": {"message": message }});
            SseEvent::default().json_data(err).unwrap()
        }
    });
    let stream_end = stream::iter([SseEvent::default().data("[DONE]")]);
    let stream = stream_default.chain(stream_msg).chain(stream_end);
    Ok(Sse::new(stream.map(Ok::<_, AnyError>))
        .keep_alive(KeepAlive::default().interval(alive_duration))
        .into_response())
}
