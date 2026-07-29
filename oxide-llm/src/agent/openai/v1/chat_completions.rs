use oxide_llm_core::mapper::openai::v1::{OpenAIChatCompletionMapper, OpenAIStreamMapper};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::tool::{ToolAdapter, ToolChoiceAdapter};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::chat_completions::chunk::ChatCompletionChunk as OpenAIStreamChunk;
use oxide_llm_proto::openai::v1::chat_completions::request::{
    ChatCompletionMessage, ChatCompletionRequest, StreamOptions,
};
use oxide_llm_proto::openai::v1::chat_completions::response::ChatCompletionResponse;

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::{
    ChatCompletionsConfig, ChatCompletionsOptionalConfig, ChatCompletionsRequiredConfig,
};

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
            config: ChatCompletionsConfig::new(required),
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
    ) -> Result<ChatCompletionRequest> {
        let ConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
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
                    acc.push(OpenAIChatCompletionMapper::from_core_message(msg)?);
                    Ok(acc)
                })
                .map_err(AgentError::Mapper)?
        };

        // 2. Construct Request using Config
        // 2. 使用 Config 构建请求
        let tc = tool_choice.map(|tc| tc.to_openai());

        let mut request = self
            .config
            .clone()
            .to_request(openai_messages, tools, tc, stream, None);

        if stream && request.stream_options.is_none() {
            request.stream_options = Some(StreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            });
        }

        Ok(request)
    }
}

// Stream for OpenAI Messages.
//
// OpenAI Messages 流。

/// SSE Processor for OpenAI Chat Completion stream chunks.
///
/// OpenAI Chat Completion 流 Chunk 的 SSE 处理器。
pub struct OpenAIProcessor {
    mapper: OpenAIStreamMapper,
}

impl OpenAIProcessor {
    /// Creates a new `OpenAIProcessor`.
    ///
    /// 创建一个新的 `OpenAIProcessor`。
    pub fn new() -> Self {
        Self {
            mapper: OpenAIStreamMapper::new(),
        }
    }
}

impl Default for OpenAIProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor for OpenAIProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<DeltaMessage>>, bool) {
        let s = match std::str::from_utf8(block) {
            Ok(s) => s,
            Err(e) => return (Some(Err(AgentError::Utf8(e))), false),
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
                match serde_json::from_str::<OpenAIStreamChunk>(data) {
                    Ok(chunk) => match self.mapper.map_response(chunk) {
                        Ok(delta) => {
                            chunk_to_yield = Some(Ok(delta));
                        }
                        Err(e) => return (Some(Err(AgentError::Mapper(e))), false),
                    },
                    Err(e) => return (Some(Err(AgentError::Json(e))), false),
                }
            }
        }

        if done {
            return (chunk_to_yield, true);
        }

        (chunk_to_yield, false)
    }
}

impl<T: Transport> ChatAgent for ChatCompletionsAgent<T> {
    type Stream = crate::stream::MessageStream<T::Stream, OpenAIProcessor>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<T::StreamFuture, OpenAIProcessor>
    where
        Self: 'a;

    /// Send a chat request to OpenAI.
    ///
    /// 发送聊天请求到 OpenAI。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state, false)?;

        // Send Request
        let transport_req = TransportRequest::new(
            Method::Post,
            self.config.required().endpoint().to_string(),
            request,
        );
        let response: ChatCompletionResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message =
            OpenAIChatCompletionMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to OpenAI and receive a stream of chunks.
    ///
    /// 发送聊天请求到 OpenAI 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        let request_res = self.build_request(state, true);
        let fut = request_res.map(|request| {
            let transport_req = TransportRequest::new(
                Method::Post,
                self.config.required().endpoint().to_string(),
                request,
            );
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamFuture::new(fut, OpenAIProcessor::new())
    }
}
