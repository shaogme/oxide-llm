use bytes::{Bytes, BytesMut};
use error_set::error_set;
use futures::{Stream, StreamExt};
use oxide_llm_core::mapper::MapperError;
use oxide_llm_core::message::{ChatStream, ChatStreamWrapper, DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportError, TransportRequest};
use oxide_llm_proto::claude::v1::messages::chunk::MessageStreamEvent;
use oxide_llm_proto::claude::v1::messages::request::{
    Message as ClaudeMessage, MessagesRequest, OutputConfig, SystemPrompt, ThinkingConfig, Tool,
    ToolChoice,
};
use oxide_llm_proto::claude::v1::messages::response::MessagesResponse;

error_set! {
    MessagesError := {
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

type Result<T> = std::result::Result<T, MessagesError>;

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
            tool_choice: self.optional.tool_choice,
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
            .map(|msg| msg.try_into())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(MessagesError::Mapper)?;

        // 2. Construct Request using Config
        // 2. 使用 Config 构建请求
        let request = self
            .config
            .clone()
            .to_request(claude_messages, system, tools, stream);

        Ok(request)
    }

    /// Send a chat request to Claude.
    ///
    /// 发送聊天请求到 Claude。
    pub async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state, false)?;

        // Send Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let response: MessagesResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(MessagesError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message = response.try_into().map_err(MessagesError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Claude and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Claude 并接收流式响应。
    pub async fn chat_stream(
        &self,
        state: ConversationState,
    ) -> Result<ChatStreamWrapper<MessagesError>> {
        let request = self.build_request(state, true)?;

        // Send Stream Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);
        let stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(MessagesError::Transport)?;

        // Parse SSE Stream
        let parsed_stream = parse_sse_stream(stream);
        Ok(ChatStream::new(Box::pin(parsed_stream)))
    }
}

fn parse_sse_stream(
    stream: futures::stream::BoxStream<'static, std::result::Result<Bytes, TransportError>>,
) -> impl Stream<Item = Result<DeltaMessage>> {
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
                            return Some((Err(MessagesError::Utf8(e)), (stream, buffer)));
                        }
                    };

                    let mut chunk_to_yield = None;
                    let mut stop = false;

                    for line in s.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let data = data.trim();

                            // Parse JSON
                            match serde_json::from_str::<MessageStreamEvent>(data) {
                                Ok(event) => {
                                    // Check if this event stops the stream
                                    if matches!(&event, MessageStreamEvent::MessageStop) {
                                        stop = true;
                                    }
                                    if let MessageStreamEvent::Error { .. } = &event {
                                        stop = true;
                                    }

                                    // Convert to DeltaMessage
                                    match event.try_into() {
                                        Ok(delta) => {
                                            chunk_to_yield = Some(delta);
                                        }
                                        Err(e) => {
                                            // Ignore explicitly ignored events (Ping, Stop, etc.)
                                            if matches!(e, MapperError::IgnoredEvent { .. }) {
                                                // Continue loop, do nothing
                                            } else {
                                                return Some((
                                                    Err(MessagesError::Mapper(e)),
                                                    (stream, buffer),
                                                ));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Some((Err(MessagesError::Json(e)), (stream, buffer)));
                                }
                            }
                        }
                    }

                    if stop {
                        if let Some(chunk) = chunk_to_yield {
                            return Some((Ok(chunk), (stream, buffer)));
                        }
                        return None;
                    }

                    if let Some(chunk) = chunk_to_yield {
                        return Some((Ok(chunk), (stream, buffer)));
                    } else {
                        // Keep-alive or ignored event, continue loop
                        continue;
                    }
                }

                // Read more
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.extend_from_slice(&chunk);
                    }
                    Some(Err(e)) => {
                        return Some((Err(MessagesError::Transport(e)), (stream, buffer)));
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
