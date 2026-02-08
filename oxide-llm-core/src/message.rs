use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Abstract message structure compatible with various Protocols.
///
/// Designed to be compatible with mainstream LLM protocols such as OpenAI, Claude, and Gemini.
/// System Prompts are not included in this structure and should be handled separately at the Agent level.
///
/// 跨协议通用的消息抽象。
/// 旨在兼容 OpenAI, Claude, Gemini 等主流 LLM 协议。
/// System Prompt 不包含在此结构中，应在 Agent 层面单独处理。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The role of the message sender.
    ///
    /// 消息发送者角色。
    pub role: Role,
    /// The content parts of the message, supporting multimode and tool calls.
    ///
    /// 消息内容部分，支持多模态和工具调用。
    pub content: Vec<ContentPart>,
    /// The name of the sender (supported by OpenAI).
    ///
    /// 发送者名称 (OpenAI 支持)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Message Role.
///
/// 消息角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User role.
    ///
    /// 用户。
    User,
    /// Assistant role (Model).
    ///
    /// 模型助手。
    Assistant,
    /// Tool role (Used to carry tool execution results).
    ///
    /// 工具 (用于承载工具执行结果)。
    Tool,
}

/// Components of the message content.
///
/// 消息内容的组成部分。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content.
    ///
    /// 文本内容。
    Text { text: String },

    /// Image content.
    ///
    /// 图片内容。
    Image(Image),

    /// Audio content.
    ///
    /// 音频内容。
    Audio(Audio),

    /// Tool call request (Usually appears in Assistant messages).
    ///
    /// 工具调用请求 (通常出现在 Assistant 消息中)。
    ToolCall(crate::tool::ToolCall),

    /// Tool call result (Usually appears in Tool messages).
    ///
    /// 工具调用结果 (通常出现在 Tool 消息中)。
    ToolResult(crate::tool::ToolResult),

    /// Content refused by the model (OpenAI specific).
    ///
    /// 模型拒绝执行的内容 (OpenAI 特有)。
    Refusal { refusal: String },

    /// JSON content.
    ///
    /// JSON 内容。
    Json(serde_json::Value),

    /// Reasoning content.
    ///
    /// 推理/思维链内容。
    Reasoning {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

/// Unified Image structure.
///
/// 统一的图片结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// Image data source.
    ///
    /// 图片数据源。
    pub source: ImageSource,
    /// MIME type (e.g., image/jpeg), usually required for Base64 data.
    ///
    /// MIME 类型 (如 image/jpeg)，对于 Base64 数据通常是必须的。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Image detail level (OpenAI: low, high, auto).
    ///
    /// 图片细节水平 (OpenAI: low, high, auto)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ImageSource {
    /// Remote URL.
    ///
    /// 远程 URL。
    Url { url: String },
    /// Base64 encoded data.
    ///
    /// Base64 编码数据。
    Base64 { data: String },
}

/// Unified Audio structure.
///
/// 统一的音频结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Audio {
    /// Base64 encoded audio data.
    ///
    /// Base64 编码的音频数据。
    pub data: String,
    /// Format (e.g., wav, mp3).
    ///
    /// 格式 (如 wav, mp3)。
    pub format: String,
}

impl Message {
    /// Create a new User message with text content.
    ///
    /// 创建带有文本内容的新 User 消息。
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text { text: text.into() }],
            name: None,
        }
    }

    /// Create a new Assistant message with text content.
    ///
    /// 创建带有文本内容的新 Assistant 消息。
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text { text: text.into() }],
            name: None,
        }
    }
}

/// Message history management.
///
/// 消息历史管理。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MessageHistory {
    /// Message list.
    ///
    /// 消息列表。
    pub messages: Vec<Message>,
}

impl MessageHistory {
    /// Create a new MessageHistory.
    ///
    /// 创建一个新的 MessageHistory。
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Add a message to history.
    ///
    /// 添加一条消息到历史。
    pub fn add(&mut self, message: Message) {
        self.messages.push(message);
    }
}

/// 增量消息结构，用于流式响应。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DeltaMessage {
    /// 角色通常只在第一个包出现，后续可能为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    /// 消息内容的增量部分。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<DeltaContentPart>>,

    /// 发送者名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 结束原因 (通常在流的最后出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,

    /// Token 使用情况 (可能在流的开始或结束出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// 增量消息内容部分。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeltaContentPart {
    /// 文本增量。
    Text { index: u32, text: String },

    /// 推理/思维链内容增量 (对应 Claude Thinking Block)。
    Reasoning {
        index: u32,
        text: String,
        /// 可选：思维块的签名/验签数据。
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// 工具调用增量。
    ToolCall(DeltaToolCall),

    /// 拒绝内容增量 (OpenAI)。
    Refusal { refusal: String },
}

/// 增量工具调用。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaToolCall {
    /// 对应 Message 中 ToolCall 列表的索引。
    pub index: u32,

    /// 工具 ID (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 工具类型 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// 函数信息增量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunction>,
}

/// 增量函数信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaFunction {
    /// 函数名 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 参数片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// 结束原因。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// 模型自然停止生成。
    Stop,
    /// 达到最大 Token 限制。
    Length,
    /// 模型请求调用工具。
    ToolCalls,
    /// 内容被安全过滤器拦截。
    ContentFilter,
    /// 其他原因。
    Other(String),
}

/// Token 使用量统计。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

// =================================================================================================
// Message Assembler & Stream Wrapper
// =================================================================================================

/// Event emitted by ChatStream.
///
/// ChatStream 产生的事件。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamEvent {
    /// Conversation start.
    ///
    /// 对话开始。
    Start { role: Role, name: Option<String> },

    /// Text stream chunk.
    ///
    /// 文本流片段。
    Text { text: String },

    /// Reasoning stream chunk.
    ///
    /// 思考/推理流片段。
    Reasoning { text: String },

    /// Tool call start chunk.
    ///
    /// 工具调用开始。
    ToolCallStart {
        index: u32,
        id: Option<String>,
        #[serde(rename = "tool_type")]
        r#type: Option<String>,
        name: Option<String>,
    },

    /// Tool call complete.
    ///
    /// 工具调用完成。
    ToolCallFinished(crate::tool::ToolCall),

    /// Response finished.
    ///
    /// 响应结束。
    Finished {
        usage: Option<Usage>,
        finish_reason: Option<FinishReason>,
    },
}

impl FromIterator<ChatStreamEvent> for Message {
    fn from_iter<T: IntoIterator<Item = ChatStreamEvent>>(iter: T) -> Self {
        let mut role = Role::Assistant;
        let mut name = None;
        let mut content = Vec::new();

        for event in iter {
            match event {
                ChatStreamEvent::Start { role: r, name: n } => {
                    role = r;
                    name = n;
                }
                ChatStreamEvent::Text { text } => match content.last_mut() {
                    Some(ContentPart::Text { text: current }) => {
                        current.push_str(&text);
                    }
                    _ => {
                        content.push(ContentPart::Text { text });
                    }
                },
                ChatStreamEvent::Reasoning { text } => match content.last_mut() {
                    Some(ContentPart::Reasoning { text: current, .. }) => {
                        current.push_str(&text);
                    }
                    _ => {
                        content.push(ContentPart::Reasoning {
                            text,
                            signature: None,
                        });
                    }
                },
                ChatStreamEvent::ToolCallFinished(tool_call) => {
                    content.push(ContentPart::ToolCall(tool_call));
                }
                _ => {}
            }
        }

        Message {
            role,
            content,
            name,
        }
    }
}

/// Helper struct to assemble a complete Message from DeltaMessages.
///
/// 用于将多个 DeltaMessage 组装成完整 Message 的辅助结构。
#[derive(Debug, Clone, Default)]
pub struct MessageAssembler {
    role: Option<Role>,
    name: Option<String>,
    // Use BTreeMap to keep content parts ordered by index
    content_parts: std::collections::BTreeMap<u32, AssembledPart>,
    usage: Option<Usage>,
    finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone)]
enum AssembledPart {
    Text(String),
    Reasoning {
        text: String,
        signature: Option<String>,
    },
    // Tool calls are complex as they have internal structure (id, type, name, args)
    ToolCall {
        _index: u32,
        id: Option<String>,
        r#type: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    // Other types like Audio/Image are usually not streamed in deltas this way yet, but reserved
}

impl MessageAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a delta message.
    ///
    /// 添加一个增量消息。
    pub fn add(&mut self, delta: DeltaMessage) {
        if let Some(role) = delta.role {
            self.role = Some(role);
        }
        if let Some(name) = delta.name {
            self.name = Some(name);
        }
        if let Some(reason) = delta.finish_reason {
            self.finish_reason = Some(reason);
        }
        if let Some(usage) = delta.usage {
            // Usually usage comes at the end, overwrite is fine or accumulate?
            // OpenAI sends separate usage chunk.
            self.usage = Some(usage);
        }

        if let Some(content) = delta.content {
            for part in content {
                match part {
                    DeltaContentPart::Text { index, text } => {
                        let entry = self
                            .content_parts
                            .entry(index)
                            .or_insert(AssembledPart::Text(String::new()));
                        if let AssembledPart::Text(current_text) = entry {
                            current_text.push_str(&text);
                        }
                    }
                    DeltaContentPart::Reasoning {
                        index,
                        text,
                        signature,
                    } => {
                        let entry =
                            self.content_parts
                                .entry(index)
                                .or_insert(AssembledPart::Reasoning {
                                    text: String::new(),
                                    signature: None,
                                });
                        if let AssembledPart::Reasoning {
                            text: current_text,
                            signature: current_sig,
                        } = entry
                        {
                            current_text.push_str(&text);
                            if let Some(sig) = signature {
                                *current_sig = Some(sig);
                            }
                        }
                    }
                    DeltaContentPart::ToolCall(tool_call) => {
                        let entry = self.content_parts.entry(tool_call.index).or_insert(
                            AssembledPart::ToolCall {
                                _index: tool_call.index,
                                id: None,
                                r#type: None,
                                name: None,
                                arguments: String::new(),
                            },
                        );

                        if let AssembledPart::ToolCall {
                            id,
                            r#type,
                            name,
                            arguments,
                            ..
                        } = entry
                        {
                            if let Some(tid) = tool_call.id {
                                *id = Some(tid);
                            }
                            if let Some(tty) = tool_call.r#type {
                                *r#type = Some(tty);
                            }
                            if let Some(func) = tool_call.function {
                                if let Some(fname) = func.name {
                                    *name = Some(fname);
                                }
                                if let Some(fargs) = func.arguments {
                                    arguments.push_str(&fargs);
                                }
                            }
                        }
                    }
                    DeltaContentPart::Refusal { .. } => {
                        // Handle refusal if needed
                    }
                }
            }
        }
    }

    /// Build the complete Message.
    ///
    /// 构建完整的 Message。
    pub fn build(self) -> Message {
        let content = self
            .content_parts
            .into_values()
            .filter_map(|part| match part {
                AssembledPart::Text(text) => Some(ContentPart::Text { text }),
                AssembledPart::Reasoning { text, signature } => {
                    Some(ContentPart::Reasoning { text, signature })
                }
                AssembledPart::ToolCall {
                    id,
                    name,
                    arguments,
                    ..
                } => {
                    // Try to parse arguments as JSON
                    let args_value: serde_json::Value = serde_json::from_str(&arguments)
                        .unwrap_or(serde_json::Value::String(arguments));

                    Some(ContentPart::ToolCall(crate::tool::ToolCall {
                        id: id.unwrap_or_default(),
                        name: name.unwrap_or_default(),
                        arguments: args_value,
                    }))
                }
            })
            .collect();

        Message {
            role: self.role.unwrap_or(Role::Assistant),
            content,
            name: self.name,
        }
    }

    pub fn usage(&self) -> Option<Usage> {
        self.usage.clone()
    }

    pub fn finish_reason(&self) -> Option<FinishReason> {
        self.finish_reason.clone()
    }

    /// Get a specific tool call by index.
    pub fn get_tool_call(&self, index: u32) -> Option<crate::tool::ToolCall> {
        if let Some(AssembledPart::ToolCall {
            id,
            name,
            arguments,
            ..
        }) = self.content_parts.get(&index)
        {
            let args_value: serde_json::Value = serde_json::from_str(arguments)
                .unwrap_or(serde_json::Value::String(arguments.clone()));

            Some(crate::tool::ToolCall {
                id: id.clone().unwrap_or_default(),
                name: name.clone().unwrap_or_default(),
                arguments: args_value,
            })
        } else {
            None
        }
    }

    /// Get all tool call indices.
    pub fn get_tool_call_indices(&self) -> Vec<u32> {
        self.content_parts
            .iter()
            .filter_map(|(idx, part)| match part {
                AssembledPart::ToolCall { .. } => Some(*idx),
                _ => None,
            })
            .collect()
    }
}

pub type ChatStreamWrapper<'a, E> =
    ChatStream<futures::stream::BoxStream<'a, Result<DeltaMessage, E>>, E>;

/// A stream wrapper that converts DeltaMessages into high-level ChatStreamEvents
/// and automatically assembles the final Message.
///
/// 一个流包装器，将 DeltaMessages 转换为高级 ChatStreamEvents，并自动组装最终的 Message。
pub struct ChatStream<S, E> {
    stream: S,
    assembler: MessageAssembler,
    pending_events: VecDeque<ChatStreamEvent>,
    stream_finished: bool,
    start_event_emitted: bool,
    finished_event_emitted: bool,
    emitted_tool_starts: std::collections::HashSet<u32>,
    current_tool_index: Option<u32>,
    emitted_tool_finishes: std::collections::HashSet<u32>,
    _marker: std::marker::PhantomData<E>,
}

impl<S, E> ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            assembler: MessageAssembler::new(),
            pending_events: VecDeque::new(),
            stream_finished: false,
            start_event_emitted: false,
            finished_event_emitted: false,
            emitted_tool_starts: std::collections::HashSet::new(),
            current_tool_index: None,
            emitted_tool_finishes: std::collections::HashSet::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Handles transition between tool states.
    /// If current_tool_index changes, emits ToolCallFinished for the previous tool.
    fn transition_tool_state(&mut self, new_index: Option<u32>) {
        if let Some(current_idx) = self.current_tool_index {
            if Some(current_idx) != new_index {
                // Previous tool finished
                if !self.emitted_tool_finishes.contains(&current_idx) {
                    if let Some(tool_call) = self.assembler.get_tool_call(current_idx) {
                        self.pending_events
                            .push_back(ChatStreamEvent::ToolCallFinished(tool_call));
                        self.emitted_tool_finishes.insert(current_idx);
                    }
                }
            }
        }
        self.current_tool_index = new_index;
    }
}

impl<S, E> Stream for ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Unpin,
    E: std::fmt::Debug + Unpin,
{
    type Item = Result<ChatStreamEvent, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // 1. Drain pending events first
            if let Some(event) = this.pending_events.pop_front() {
                return Poll::Ready(Some(Ok(event)));
            }

            // 2. If finished event emitted, we are done
            if this.finished_event_emitted {
                return Poll::Ready(None);
            }

            // 3. If stream finished but we haven't emitted Finished event, do it now
            if this.stream_finished {
                this.finished_event_emitted = true;

                // Ensure the last active tool is finished
                this.transition_tool_state(None);

                // Emit all completed tool calls that haven't been emitted yet (Just in case)
                let tool_indices = this.assembler.get_tool_call_indices();
                for index in tool_indices {
                    if !this.emitted_tool_finishes.contains(&index) {
                        if let Some(tool_call) = this.assembler.get_tool_call(index) {
                            this.pending_events
                                .push_back(ChatStreamEvent::ToolCallFinished(tool_call));
                            this.emitted_tool_finishes.insert(index);
                        }
                    }
                }

                let usage = this.assembler.usage();
                let finish_reason = this.assembler.finish_reason();

                return Poll::Ready(Some(Ok(ChatStreamEvent::Finished {
                    usage,
                    finish_reason,
                })));
            }

            // 4. Poll underlying stream
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(mut delta))) => {
                    // 4.1 Extract content and update Assembler Metadata first
                    // We take the content out, leaving None in delta, so delta becomes pure metadata container
                    let content = delta.content.take();
                    this.assembler.add(delta);

                    // 4.2 Emit Start Event if not yet emitted
                    if !this.start_event_emitted {
                        this.start_event_emitted = true;
                        this.pending_events.push_back(ChatStreamEvent::Start {
                            role: this.assembler.role.unwrap_or(Role::Assistant),
                            name: this.assembler.name.clone(),
                        });
                    }

                    // 4.3 Process Content
                    // We filter out Text/Reasoning to avoid double buffering in Assembler
                    let mut tool_content_buffer = Vec::new();

                    if let Some(parts) = content {
                        for part in parts {
                            // Determine active tool index for this part to manage transitions
                            let part_tool_idx = if let DeltaContentPart::ToolCall(t) = &part {
                                Some(t.index)
                            } else {
                                None
                            };

                            this.transition_tool_state(part_tool_idx);

                            match part {
                                DeltaContentPart::Text { text, .. } => {
                                    if !text.is_empty() {
                                        this.pending_events
                                            .push_back(ChatStreamEvent::Text { text });
                                    }
                                    // Drop text part, do not store in assembler
                                }
                                DeltaContentPart::Reasoning { text, .. } => {
                                    if !text.is_empty() {
                                        this.pending_events
                                            .push_back(ChatStreamEvent::Reasoning { text });
                                    }
                                    // Drop reasoning part, do not store in assembler
                                }
                                DeltaContentPart::ToolCall(tool) => {
                                    if !this.emitted_tool_starts.contains(&tool.index) {
                                        this.emitted_tool_starts.insert(tool.index);
                                        this.pending_events.push_back(
                                            ChatStreamEvent::ToolCallStart {
                                                index: tool.index,
                                                id: tool.id.clone(),
                                                r#type: tool.r#type.clone(),
                                                name: tool
                                                    .function
                                                    .as_ref()
                                                    .and_then(|f| f.name.clone()),
                                            },
                                        );
                                    }
                                    // Keep ToolCall for assembler (needed for ToolCallFinished)
                                    tool_content_buffer.push(DeltaContentPart::ToolCall(tool));
                                }
                                other => {
                                    // Keep other parts (e.g. Refusal) just in case
                                    tool_content_buffer.push(other);
                                }
                            }
                        }
                    } else {
                        // No content, ensure tool state transitions to None (finish previous tool if any)
                        this.transition_tool_state(None);
                    }

                    // 4.4 Feed Tool content to Assembler
                    if !tool_content_buffer.is_empty() {
                        let content_delta = DeltaMessage {
                            role: None,
                            name: None,
                            content: Some(tool_content_buffer),
                            finish_reason: None,
                            usage: None,
                        };
                        this.assembler.add(content_delta);
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    this.stream_finished = true;
                    // Continue loop to hit step 3
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
