use bytes::{Bytes, BytesMut};
use error_set::error_set;
use futures::{Stream, StreamExt};
use oxide_llm_core::mapper::MapperError;
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportError, TransportRequest};
use oxide_llm_proto::openai::v1::chat_completions::chunk::ChatCompletionChunk;
use oxide_llm_proto::openai::v1::chat_completions::request::{
    AudioOptions, ChatCompletionMessage, ChatCompletionRequest, PredictionContent, ResponseFormat,
    Stop, StreamOptions, WebSearchOptions,
};
use oxide_llm_proto::openai::v1::chat_completions::response::ChatCompletionResponse;
use oxide_llm_proto::openai::v1::{FunctionDefinition, Tool, ToolChoice};
use std::collections::HashMap;

error_set! {
    ChatCompletionsError := {
        #[display("Transport error: {0}")]
        Transport(TransportError),
        #[display("Mapper conversion error: {0}")]
        Mapper(MapperError),
        #[display("JSON error: {0}")]
        Json(serde_json::Error),
        #[display("UTF-8 error: {0}")]
        Utf8(std::str::Utf8Error),
    }
}

/// Configuration for OpenAI Chat Completions Agent (Required).
///
/// OpenAI Chat Completions 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct ChatCompletionsRequiredConfig {
    pub model: String,
    pub endpoint: String,
}

/// Configuration for OpenAI Chat Completions Agent (Optional).
///
/// OpenAI Chat Completions 代理配置 (选填)。
/// 包含了除 `messages` 和 `model` 之外的所有 `ChatCompletionRequest` 可选参数。
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionsOptionalConfig {
    pub frequency_penalty: Option<f32>,
    pub logit_bias: Option<HashMap<String, f32>>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u8>,
    pub max_tokens: Option<u32>,
    pub max_completion_tokens: Option<u32>,
    pub n: Option<u8>,
    pub modalities: Option<Vec<String>>,
    pub prediction: Option<PredictionContent>,
    pub audio: Option<AudioOptions>,
    pub presence_penalty: Option<f32>,
    pub response_format: Option<ResponseFormat>,
    pub seed: Option<i64>,
    pub service_tier: Option<String>,
    pub stop: Option<Stop>,
    pub store: Option<bool>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub user: Option<String>,
    pub function_call: Option<serde_json::Value>,
    pub functions: Option<Vec<FunctionDefinition>>,
    pub web_search_options: Option<WebSearchOptions>,
    pub verbosity: Option<String>,
    pub reasoning_effort: Option<String>,
}

/// Configuration for OpenAI Chat Completions Agent.
///
/// OpenAI Chat Completions 代理配置。
#[derive(Debug, Clone)]
pub struct ChatCompletionsConfig {
    pub required: ChatCompletionsRequiredConfig,
    pub optional: ChatCompletionsOptionalConfig,
}

impl ChatCompletionsConfig {
    /// Convert Config to ChatCompletionRequest with provided messages.
    ///
    /// 将配置转换为 ChatCompletionRequest，并填入消息。
    pub fn to_request(
        self,
        messages: Vec<ChatCompletionMessage>,
        tools: Option<Vec<Tool>>,
        is_stream: bool,
        stream_options: Option<StreamOptions>,
    ) -> ChatCompletionRequest {
        ChatCompletionRequest {
            messages,
            model: self.required.model,
            frequency_penalty: self.optional.frequency_penalty,
            logit_bias: self.optional.logit_bias,
            logprobs: self.optional.logprobs,
            top_logprobs: self.optional.top_logprobs,
            max_tokens: self.optional.max_tokens,
            max_completion_tokens: self.optional.max_completion_tokens,
            n: self.optional.n,
            modalities: self.optional.modalities,
            prediction: self.optional.prediction,
            audio: self.optional.audio,
            presence_penalty: self.optional.presence_penalty,
            response_format: self.optional.response_format,
            seed: self.optional.seed,
            service_tier: self.optional.service_tier,
            stop: self.optional.stop,
            store: self.optional.store,
            stream: Some(is_stream),
            stream_options,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            tools,
            tool_choice: self.optional.tool_choice,
            parallel_tool_calls: self.optional.parallel_tool_calls,
            user: self.optional.user,
            function_call: self.optional.function_call,
            functions: self.optional.functions,
            web_search_options: self.optional.web_search_options,
            verbosity: self.optional.verbosity,
            reasoning_effort: self.optional.reasoning_effort,
        }
    }
}

/// OpenAI Chat Completions Agent.
///
/// OpenAI Chat Completions 代理。
/// 负责处理与 OpenAI Chat API 的基本交互，无需维护状态。
#[derive(Clone)]
pub struct ChatCompletionsAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The configuration for the chat completions request.
    ///
    /// 聊天补全请求的配置。
    config: ChatCompletionsConfig,
}

impl<T: Transport> ChatCompletionsAgent<T> {
    /// Create a new ChatCompletionsAgent.
    ///
    /// 创建一个新的 ChatCompletionsAgent。
    pub fn new(transport: T, required: ChatCompletionsRequiredConfig) -> Self {
        Self {
            transport,
            config: ChatCompletionsConfig {
                required,
                optional: ChatCompletionsOptionalConfig::default(),
            },
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: ChatCompletionsConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a ChatCompletionRequest from the conversation state.
    ///
    /// 根据对话状态构建 ChatCompletionRequest。
    fn build_request(
        &self,
        state: ConversationState,
        stream: bool,
    ) -> Result<ChatCompletionRequest, ChatCompletionsError> {
        let ConversationState {
            system_prompt,
            messages,
            tools,
        } = state;

        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.into_iter().map(|t| t.to_openai()).collect())
        };

        // 1. Convert Core Messages to OpenAI Messages
        // 1. 将核心消息转换为 OpenAI 消息
        let openai_messages: Vec<ChatCompletionMessage> = {
            let initial_messages = match system_prompt {
                Some(prompt) => {
                    vec![ChatCompletionMessage::System {
                        content: prompt,
                        name: None,
                    }]
                }
                None => Vec::new(),
            };

            messages
                .into_iter()
                .try_fold(initial_messages, |mut acc, msg| {
                    acc.push(msg.try_into()?);
                    Ok(acc)
                })
                .map_err(ChatCompletionsError::Mapper)?
        };

        // 2. Construct Request using Config
        // 2. 使用 Config 构建请求
        let mut request = self
            .config
            .clone()
            .to_request(openai_messages, tools, stream, None);

        if stream && request.stream_options.is_none() {
            request.stream_options = Some(StreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            });
        }

        Ok(request)
    }

    /// Send a chat request to OpenAI.
    ///
    /// 发送聊天请求到 OpenAI。
    pub async fn chat(&self, state: ConversationState) -> Result<Message, ChatCompletionsError> {
        let request = self.build_request(state, false)?;

        // Send Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let response: ChatCompletionResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(ChatCompletionsError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message = response.try_into().map_err(ChatCompletionsError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to OpenAI and receive a stream of chunks.
    ///
    /// 发送聊天请求到 OpenAI 并接收流式响应。
    pub async fn chat_stream(
        &self,
        state: ConversationState,
    ) -> Result<impl Stream<Item = Result<DeltaMessage, ChatCompletionsError>>, ChatCompletionsError>
    {
        let request = self.build_request(state, true)?;

        // Send Stream Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(ChatCompletionsError::Transport)?;

        // Parse SSE Stream
        Ok(parse_sse_stream(stream))
    }
}

fn parse_sse_stream(
    stream: futures::stream::BoxStream<'static, Result<Bytes, TransportError>>,
) -> impl Stream<Item = Result<DeltaMessage, ChatCompletionsError>> {
    futures::stream::unfold(
        (stream, BytesMut::new()),
        |(mut stream, mut buffer)| async move {
            loop {
                // Check if buffer contains double newline
                if let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                    // Found an event block
                    let block = buffer.split_to(pos + 2); // Remove block from buffer

                    let s = match std::str::from_utf8(&block) {
                        Ok(s) => s,
                        Err(e) => {
                            return Some((Err(ChatCompletionsError::Utf8(e)), (stream, buffer)));
                        }
                    };

                    let mut chunk_to_yield = None;
                    let mut done = false;

                    for line in s.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let data = data.trim();
                            if data == "[DONE]" {
                                done = true;
                                break;
                            }
                            // Parse JSON
                            match serde_json::from_str::<ChatCompletionChunk>(data) {
                                Ok(chunk) => {
                                    // Convert to DeltaMessage
                                    match chunk.try_into() {
                                        Ok(delta) => {
                                            chunk_to_yield = Some(delta);
                                        }
                                        Err(e) => {
                                            return Some((
                                                Err(ChatCompletionsError::Mapper(e)),
                                                (stream, buffer),
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Some((
                                        Err(ChatCompletionsError::Json(e)),
                                        (stream, buffer),
                                    ));
                                }
                            }
                        }
                    }

                    if done {
                        return None;
                    }

                    if let Some(chunk) = chunk_to_yield {
                        return Some((Ok(chunk), (stream, buffer)));
                    } else {
                        // Keep-alive or empty block, continue loop
                        continue;
                    }
                }

                // Read more
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.extend_from_slice(&chunk);
                    }
                    Some(Err(e)) => {
                        return Some((Err(ChatCompletionsError::Transport(e)), (stream, buffer)));
                    }
                    None => {
                        // EOF
                        return None;
                    }
                }
            }
        },
    )
}
