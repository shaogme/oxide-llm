use oxide_llm_core::mapper::openai::v1::{OpenAIChatCompletionMapper, OpenAIStreamMapper};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::{ConversationState, RawConversationState};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::chat_completions::{
    Tool as OpenAIChatCompletionsTool, ToolChoice as OpenAIChatCompletionsToolChoice,
    chunk::ChatCompletionChunk as OpenAIStreamChunk,
    request::{ChatCompletionMessage, ChatCompletionRequest, StreamOptions},
    response::ChatCompletionResponse,
};

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

    /// Build a ChatCompletionRequest from the raw conversation state.
    ///
    /// 根据底层原始对话状态构建 ChatCompletionRequest。
    fn build_request(
        &self,
        state: RawConversationState<
            ChatCompletionMessage,
            OpenAIChatCompletionsTool,
            OpenAIChatCompletionsToolChoice,
        >,
        stream: bool,
    ) -> Result<ChatCompletionRequest> {
        let RawConversationState {
            system_prompt,
            mut messages,
            tools,
            tool_choice,
        } = state;

        if let Some(prompt) = system_prompt {
            messages.insert(
                0,
                ChatCompletionMessage::System {
                    content: prompt,
                    name: None,
                },
            );
        }

        let tools = if tools.is_empty() { None } else { Some(tools) };

        let mut request = self
            .config
            .clone()
            .to_request(messages, tools, tool_choice, stream, None);

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

/// SSE Processor for OpenAI Chat Completion raw stream chunks.
///
/// OpenAI Chat Completion 裸 Chunk 的 SSE 处理器。
pub struct RawOpenAIProcessor;

impl RawOpenAIProcessor {
    /// Creates a new `RawOpenAIProcessor`.
    ///
    /// 创建一个新的 `RawOpenAIProcessor`。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawOpenAIProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor<OpenAIStreamChunk> for RawOpenAIProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<OpenAIStreamChunk>>, bool) {
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
                    Ok(chunk) => {
                        chunk_to_yield = Some(Ok(chunk));
                    }
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

impl crate::stream::StreamMapper<OpenAIStreamChunk> for OpenAIStreamMapper {
    fn map_item(&mut self, raw: OpenAIStreamChunk) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for ChatCompletionsAgent<T> {
    type RawInputMessage = ChatCompletionMessage;
    type RawTool = OpenAIChatCompletionsTool;
    type RawToolChoice = OpenAIChatCompletionsToolChoice;
    type RawMessage = ChatCompletionResponse;
    type RawDelta = OpenAIStreamChunk;
    type RawStream =
        crate::stream::MessageStream<T::Stream, RawOpenAIProcessor, OpenAIStreamChunk>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<
            T::StreamFuture,
            RawOpenAIProcessor,
            OpenAIStreamChunk,
        >
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<Self::RawStream, OpenAIStreamMapper, Self::RawDelta>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
            Self::ChatStreamRawFuture<'a>,
            OpenAIStreamMapper,
            Self::RawDelta,
        >
    where
        Self: 'a;

    /// Send a chat request to OpenAI and return raw response.
    ///
    /// 发送聊天请求到 OpenAI 并返回原始响应。
    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<ChatCompletionResponse> {
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

        Ok(response)
    }

    /// Send a chat request to OpenAI and receive a stream of raw chunks with configuration.
    ///
    /// 发送聊天请求到 OpenAI 并接收带有配置的原始块的流式响应。
    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        mut config: crate::ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let request_res = self.build_request(state, true);
        let fut = request_res.map(|request| {
            let transport_req = TransportRequest::new(
                Method::Post,
                self.config.required().endpoint().to_string(),
                request,
            );
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::with_hook(
            fut,
            RawOpenAIProcessor::new(),
            on_raw_delta,
        )
    }

    /// Send a chat request to OpenAI.
    ///
    /// 发送聊天请求到 OpenAI。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let raw_state = RawConversationState::try_from(state).map_err(AgentError::Mapper)?;
        let response = self.chat_raw(raw_state).await?;

        // Convert Response back to Core Message
        let core_message: Message =
            OpenAIChatCompletionMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to OpenAI and receive a stream of chunks with configuration.
    ///
    /// 发送聊天请求到 OpenAI 并接收带有配置的流式响应。
    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        mut config: crate::ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let on_delta = config.take_on_delta();
        let raw_state_res = RawConversationState::try_from(state).map_err(AgentError::Mapper);
        let raw_stream_fut = match raw_state_res {
            Ok(raw_state) => self.chat_stream_raw(raw_state),
            Err(e) => crate::stream::AgentChatStreamRawFuture::with_hook(
                Err(e),
                RawOpenAIProcessor::new(),
                None,
            ),
        };
        crate::stream::AgentChatStreamFuture::with_hooks(
            raw_stream_fut,
            OpenAIStreamMapper::new(),
            on_raw_delta,
            on_delta,
        )
    }
}
