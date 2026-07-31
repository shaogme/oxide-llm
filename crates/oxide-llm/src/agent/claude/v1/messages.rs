use oxide_llm_core::mapper::claude::v1::{
    ClaudeMessagesMapper, ClaudeMessagesStreamMapper, MessagesConversationState,
};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::claude::v1::messages::{
    MessageStreamEvent as ClaudeStreamEvent, MessagesRequest, MessagesResponse, SystemPrompt,
};

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::MessagesConfig;

/// Claude Messages Agent.
///
/// Claude Messages 代理。
/// 负责处理与 Anthropic Claude Messages API 的基本交互。
#[derive(Clone)]
pub struct MessagesAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The configuration for the messages request.
    ///
    /// Messages 请求的配置。
    config: MessagesConfig,
}

pub type MessagesAgentBuilder<T> =
    crate::agent::builder::AgentBuilder<T, MessagesConfig, MessagesAgent<T>>;

impl<T: Transport> MessagesAgentBuilder<T> {
    /// Build the `MessagesAgent`.
    ///
    /// 构建 `MessagesAgent`。
    pub fn build(self) -> Result<MessagesAgent<T>> {
        let (transport, config) = self.build_config()?;
        Ok(MessagesAgent { transport, config })
    }
}

impl<T: Transport> MessagesAgent<T> {
    /// Create a new builder for MessagesAgent.
    ///
    /// 创建 MessagesAgent 的构建器。
    pub fn builder(transport: T) -> MessagesAgentBuilder<T> {
        MessagesAgentBuilder::new(transport)
    }

    /// Build a MessagesRequest from the raw conversation state.
    ///
    /// 根据底层原始对话状态构建 MessagesRequest。
    fn build_request(
        &self,
        state: MessagesConversationState,
        stream: bool,
    ) -> Result<MessagesRequest> {
        let MessagesConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        let system = system_prompt.map(SystemPrompt::Text);
        let tools = if tools.is_empty() { None } else { Some(tools) };

        let request = self
            .config
            .clone()
            .to_request(messages, system, tools, tool_choice, stream);

        Ok(request)
    }
}

// Stream for Claude Messages.
//
// Claude Messages 流。

/// SSE Processor for Claude Messages raw stream events.
///
/// Claude Messages 裸事件的 SSE 处理器。
pub struct RawClaudeProcessor;

impl RawClaudeProcessor {
    /// Creates a new `RawClaudeProcessor`.
    ///
    /// 创建一个新的 `RawClaudeProcessor`。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawClaudeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor<ClaudeStreamEvent> for RawClaudeProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<ClaudeStreamEvent>>, bool) {
        crate::stream::parse_json_sse_block(block, |event| {
            matches!(
                event,
                ClaudeStreamEvent::MessageStop | ClaudeStreamEvent::Error { .. }
            )
        })
    }
}

impl crate::stream::StreamMapper<ClaudeStreamEvent> for ClaudeMessagesStreamMapper {
    fn map_item(&mut self, raw: ClaudeStreamEvent) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for MessagesAgent<T> {
    type RawConversationState = MessagesConversationState;
    type RawMessage = MessagesResponse;
    type RawDelta = ClaudeStreamEvent;
    type RawStream = crate::stream::MessageStream<T::Stream, RawClaudeProcessor, ClaudeStreamEvent>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<
        T::StreamFuture,
        RawClaudeProcessor,
        ClaudeStreamEvent,
    >
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<Self::RawStream, ClaudeMessagesStreamMapper, Self::RawDelta>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
        Self::ChatStreamRawFuture<'a>,
        ClaudeMessagesStreamMapper,
        Self::RawDelta,
    >
    where
        Self: 'a;

    /// Send a chat request to Claude and return raw response.
    ///
    /// 发送聊天请求到 Claude 并返回原始响应。
    async fn chat_raw(&self, state: Self::RawConversationState) -> Result<MessagesResponse> {
        let request = self.build_request(state, false)?;

        let endpoint = self.config.endpoint().unwrap_or("messages").to_string();

        // Send Request
        let transport_req = TransportRequest::new(Method::Post, endpoint, request);
        let response: MessagesResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        Ok(response)
    }

    /// Send a chat request to Claude and receive a stream of raw chunks with configuration.
    ///
    /// 发送聊天请求到 Claude 并接收带有配置的原始块的流式响应。
    fn chat_stream_raw_with<'a>(
        &'a self,
        state: Self::RawConversationState,
        mut config: crate::ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let request_res = self.build_request(state, true);
        let fut = request_res.map(|request| {
            let endpoint = self.config.endpoint().unwrap_or("messages").to_string();
            let transport_req = TransportRequest::new(Method::Post, endpoint, request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::with_hook(
            fut,
            RawClaudeProcessor::new(),
            on_raw_delta,
        )
    }

    /// Send a chat request to Claude.
    ///
    /// 发送聊天请求到 Claude。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let raw_state = MessagesConversationState::try_from(state).map_err(AgentError::Mapper)?;
        let response = self.chat_raw(raw_state).await?;

        // Convert Response back to Core Message
        let core_message: Message =
            ClaudeMessagesMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Claude and receive a stream of chunks with configuration.
    ///
    /// 发送聊天请求到 Claude 并接收带有配置的流式响应。
    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        mut config: crate::ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let on_delta = config.take_on_delta();
        let raw_state_res = MessagesConversationState::try_from(state).map_err(AgentError::Mapper);
        let raw_stream_fut = match raw_state_res {
            Ok(raw_state) => self.chat_stream_raw(raw_state),
            Err(e) => crate::stream::AgentChatStreamRawFuture::with_hook(
                Err(e),
                RawClaudeProcessor::new(),
                None,
            ),
        };
        crate::stream::AgentChatStreamFuture::with_hooks(
            raw_stream_fut,
            ClaudeMessagesStreamMapper::new(),
            on_raw_delta,
            on_delta,
        )
    }
}
