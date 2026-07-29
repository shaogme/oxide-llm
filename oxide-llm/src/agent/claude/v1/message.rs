use oxide_llm_core::mapper::claude::v1::{ClaudeMapper, ClaudeStreamMapper};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::claude::v1::messages::chunk::MessageStreamEvent as ClaudeStreamEvent;
use oxide_llm_proto::claude::v1::messages::request::{
    Message as ClaudeMessage, MessagesRequest, SystemPrompt,
};
use oxide_llm_proto::claude::v1::messages::response::MessagesResponse;

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::{MessagesConfig, MessagesOptionalConfig, MessagesRequiredConfig};

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

impl<T: Transport> MessagesAgent<T> {
    /// Create a new MessagesAgent.
    ///
    /// 创建一个新的 MessagesAgent。
    pub fn new(transport: T, required: MessagesRequiredConfig) -> Self {
        Self {
            transport,
            config: MessagesConfig::new(required),
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: MessagesConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a MessagesRequest from the conversation state.
    ///
    /// 根据对话状态构建 MessagesRequest。
    fn build_request(&self, state: ConversationState, stream: bool) -> Result<MessagesRequest> {
        let ConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        let system = system_prompt.map(SystemPrompt::Text);

        let tools = if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .iter()
                    .map(ClaudeMapper::tool_to_claude_tool)
                    .collect(),
            )
        };

        // 1. Convert Core Messages to Claude Messages
        // 1. 将核心消息转换为 Claude 消息
        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .map(ClaudeMapper::from_core_message)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AgentError::Mapper)?;

        // 2. Construct Request using Config
        // 2. 使用 Config 构建请求
        let tc = tool_choice.as_ref().map(ClaudeMapper::tool_choice_to_claude);

        let request = self
            .config
            .clone()
            .to_request(claude_messages, system, tools, tc, stream);

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
        let s = match std::str::from_utf8(block) {
            Ok(s) => s,
            Err(e) => return (Some(Err(AgentError::Utf8(e))), false),
        };

        let mut chunk_to_yield = None;
        let mut stop = false;

        for line in s.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();

                match serde_json::from_str::<ClaudeStreamEvent>(data) {
                    Ok(event) => {
                        if matches!(&event, ClaudeStreamEvent::MessageStop) {
                            stop = true;
                        }
                        if let ClaudeStreamEvent::Error { .. } = &event {
                            stop = true;
                        }

                        chunk_to_yield = Some(Ok(event));
                    }
                    Err(e) => return (Some(Err(AgentError::Json(e))), false),
                }
            }
        }

        (chunk_to_yield, stop)
    }
}

impl crate::stream::StreamMapper<ClaudeStreamEvent> for ClaudeStreamMapper {
    fn map_item(&mut self, raw: ClaudeStreamEvent) -> Result<Option<DeltaMessage>> {
        match self.map_response(raw) {
            Ok(delta) => Ok(Some(delta)),
            Err(e) => {
                if matches!(
                    e,
                    oxide_llm_core::mapper::MapperError::IgnoredEvent { .. }
                ) {
                    Ok(None)
                } else {
                    Err(AgentError::Mapper(e))
                }
            }
        }
    }
}

impl<T: Transport> ChatAgent for MessagesAgent<T> {
    type RawMessage = MessagesResponse;
    type RawDelta = ClaudeStreamEvent;
    type RawStream =
        crate::stream::MessageStream<T::Stream, RawClaudeProcessor, ClaudeStreamEvent>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<T::StreamFuture, RawClaudeProcessor, ClaudeStreamEvent>
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<Self::RawStream, ClaudeStreamMapper, Self::RawDelta>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
            Self::ChatStreamRawFuture<'a>,
            ClaudeStreamMapper,
            Self::RawDelta,
        >
    where
        Self: 'a;

    /// Send a chat request to Claude and return raw response.
    ///
    /// 发送聊天请求到 Claude 并返回原始响应。
    async fn chat_raw(&self, state: ConversationState) -> Result<MessagesResponse> {
        let request = self.build_request(state, false)?;

        // Send Request
        let transport_req = TransportRequest::new(
            Method::Post,
            self.config.required().endpoint().to_string(),
            request,
        );
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
        state: ConversationState,
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
            RawClaudeProcessor::new(),
            on_raw_delta,
        )
    }

    /// Send a chat request to Claude.
    ///
    /// 发送聊天请求到 Claude。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let response = self.chat_raw(state).await?;

        // Convert Response back to Core Message
        let core_message: Message =
            ClaudeMapper::to_core_message(response).map_err(AgentError::Mapper)?;

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
        crate::stream::AgentChatStreamFuture::with_hooks(
            self.chat_stream_raw(state),
            ClaudeStreamMapper::new(),
            on_raw_delta,
            on_delta,
        )
    }
}
