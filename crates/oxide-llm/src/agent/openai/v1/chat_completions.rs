use oxide_llm_core::mapper::openai::v1::{
    ChatCompletionsConversationState, OpenAIChatCompletionMapper, OpenAIStreamMapper,
};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::chat_completions::{
    chunk::ChatCompletionChunk as OpenAIStreamChunk,
    request::{ChatCompletionMessage, ChatCompletionRequest, StreamOptions},
    response::ChatCompletionResponse,
};

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::ChatCompletionsConfig;

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

pub type ChatCompletionsAgentBuilder<T> =
    crate::agent::builder::AgentBuilder<T, ChatCompletionsConfig, ChatCompletionsAgent<T>>;

impl<T: Transport> ChatCompletionsAgentBuilder<T> {
    /// Build the `ChatCompletionsAgent`.
    ///
    /// 构建 `ChatCompletionsAgent`。
    pub fn build(self) -> Result<ChatCompletionsAgent<T>> {
        let (transport, config) = self.build_config()?;
        Ok(ChatCompletionsAgent { transport, config })
    }
}

impl<T: Transport> ChatCompletionsAgent<T> {
    /// Create a new builder for ChatCompletionsAgent.
    ///
    /// 创建 ChatCompletionsAgent 的构建器。
    pub fn builder(transport: T) -> ChatCompletionsAgentBuilder<T> {
        ChatCompletionsAgentBuilder::new(transport)
    }

    /// Build a ChatCompletionRequest from the raw conversation state.
    ///
    /// 根据底层原始对话状态构建 ChatCompletionRequest。
    fn build_request(
        &self,
        state: ChatCompletionsConversationState,
        stream: bool,
    ) -> Result<ChatCompletionRequest> {
        let ChatCompletionsConversationState {
            system_prompt,
            mut messages,
            tools,
            tool_choice,
        } = state;

        if let Some(prompt) = system_prompt {
            messages.insert(
                0,
                ChatCompletionMessage::System {
                    content: prompt.into(),
                    name: None,
                },
            );
        }

        let tools = if tools.is_empty() { None } else { Some(tools) };

        let mut request =
            self.config
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
        crate::stream::parse_json_sse_block(block, |_| false)
    }
}

impl crate::stream::StreamMapper<OpenAIStreamChunk> for OpenAIStreamMapper {
    fn map_item(&mut self, raw: OpenAIStreamChunk) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for ChatCompletionsAgent<T> {
    type RawConversationState = ChatCompletionsConversationState;
    type RawMessage = ChatCompletionResponse;
    type RawDelta = OpenAIStreamChunk;
    type RawStream = crate::stream::MessageStream<T::Stream, RawOpenAIProcessor, OpenAIStreamChunk>;
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
    async fn chat_raw(&self, state: Self::RawConversationState) -> Result<ChatCompletionResponse> {
        let request = self.build_request(state, false)?;

        let endpoint = self
            .config
            .endpoint()
            .unwrap_or("chat/completions")
            .to_string();

        // Send Request
        let transport_req = TransportRequest::new(Method::Post, endpoint, request);
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
        state: Self::RawConversationState,
        mut config: crate::ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let request_res = self.build_request(state, true);
        let fut = request_res.map(|request| {
            let endpoint = self
                .config
                .endpoint()
                .unwrap_or("chat/completions")
                .to_string();
            let transport_req = TransportRequest::new(Method::Post, endpoint, request);
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
        let raw_state =
            ChatCompletionsConversationState::try_from(state).map_err(AgentError::Mapper)?;
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
        let raw_state_res =
            ChatCompletionsConversationState::try_from(state).map_err(AgentError::Mapper);
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
