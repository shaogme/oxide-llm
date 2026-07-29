use oxide_llm_core::mapper::claude::v1::{ClaudeMapper, ClaudeStreamMapper};
use oxide_llm_core::message::{ChatStream, DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::tool::{ToolAdapter, ToolChoiceAdapter};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::claude::v1::messages::chunk::MessageStreamEvent as ClaudeStreamEvent;
use oxide_llm_proto::claude::v1::messages::request::{
    Message as ClaudeMessage, MessagesRequest, OutputConfig, SystemPrompt, ThinkingConfig, Tool,
    ToolChoice,
};
use oxide_llm_proto::claude::v1::messages::response::MessagesResponse;

use crate::ChatAgent;
use crate::error::{AgentError, Result};

/// Configuration for Claude Messages Agent (Required).
///
/// Claude Messages 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct MessagesRequiredConfig {
    pub model: String,
    /// The maximum number of tokens to generate.
    ///
    /// 最大生成 token 数。
    pub max_tokens: u32,
    pub endpoint: String,
}

/// Configuration for Claude Messages Agent (Optional).
///
/// Claude Messages 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct MessagesOptionalConfig {
    pub metadata: Option<oxide_llm_proto::claude::v1::messages::request::Metadata>,
    pub stop_sequences: Option<Vec<String>>,
    pub temperature: Option<f32>,
    pub tool_choice: Option<ToolChoice>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
    pub thinking: Option<ThinkingConfig>,
    pub output_config: Option<OutputConfig>,
    pub service_tier: Option<String>,
}

/// Configuration for Claude Messages Agent.
///
/// Claude Messages 代理配置。
#[derive(Debug, Clone)]
pub struct MessagesConfig {
    pub required: MessagesRequiredConfig,
    pub optional: MessagesOptionalConfig,
}

impl MessagesConfig {
    /// Convert Config to MessagesRequest with provided messages.
    ///
    /// 将配置转换为 MessagesRequest，并填入消息。
    pub fn to_request(
        self,
        messages: Vec<ClaudeMessage>,
        system: Option<SystemPrompt>,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        stream: bool,
    ) -> MessagesRequest {
        MessagesRequest {
            model: self.required.model,
            messages,
            max_tokens: Some(self.required.max_tokens),
            system,
            metadata: self.optional.metadata,
            stop_sequences: self.optional.stop_sequences,
            stream: Some(stream),
            temperature: self.optional.temperature,
            tool_choice,
            tools,
            top_k: self.optional.top_k,
            top_p: self.optional.top_p,
            thinking: self.optional.thinking,
            output_config: self.optional.output_config,
            service_tier: self.optional.service_tier,
        }
    }
}

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
            config: MessagesConfig {
                required,
                optional: MessagesOptionalConfig::default(),
            },
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
            Some(tools.into_iter().map(|t| t.to_claude_tool()).collect())
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
        let tc = tool_choice.map(|tc| tc.to_claude());

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

/// SSE Processor for Claude Messages stream events.
///
/// Claude Messages 流事件的 SSE 处理器。
pub struct ClaudeProcessor {
    mapper: ClaudeStreamMapper,
}

impl ClaudeProcessor {
    /// Creates a new `ClaudeProcessor`.
    ///
    /// 创建一个新的 `ClaudeProcessor`。
    pub fn new() -> Self {
        Self {
            mapper: ClaudeStreamMapper::new(),
        }
    }
}

impl Default for ClaudeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor for ClaudeProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<DeltaMessage>>, bool) {
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

                        match self.mapper.map_response(event) {
                            Ok(delta) => {
                                chunk_to_yield = Some(Ok(delta));
                            }
                            Err(e) => {
                                if !matches!(
                                    e,
                                    oxide_llm_core::mapper::MapperError::IgnoredEvent { .. }
                                ) {
                                    return (Some(Err(AgentError::Mapper(e))), false);
                                }
                            }
                        }
                    }
                    Err(e) => return (Some(Err(AgentError::Json(e))), false),
                }
            }
        }

        (chunk_to_yield, stop)
    }
}

impl<T: Transport> ChatAgent for MessagesAgent<T> {
    type Stream = crate::stream::MessageStream<T::Stream, ClaudeProcessor>;

    /// Send a chat request to Claude.
    ///
    /// 发送聊天请求到 Claude。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state, false)?;

        // Send Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let response: MessagesResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message =
            ClaudeMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Claude and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Claude 并接收流式响应。
    async fn chat_stream(
        &self,
        state: ConversationState,
    ) -> Result<ChatStream<Self::Stream, AgentError>> {
        let request = self.build_request(state, true)?;

        // Send Stream Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        let message_stream = crate::stream::MessageStream::new(stream, ClaudeProcessor::new());

        Ok(ChatStream::new(message_stream))
    }
}
