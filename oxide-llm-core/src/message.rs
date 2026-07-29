use futures::{Stream, StreamExt};
use ref_str::StaticRefStr;
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
    pub name: Option<StaticRefStr>,
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
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<StaticRefStr>,
    },

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
    Refusal { refusal: StaticRefStr },

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
        signature: Option<StaticRefStr>,
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
    pub media_type: Option<StaticRefStr>,
    /// Image detail level (OpenAI: low, high, auto).
    ///
    /// 图片细节水平 (OpenAI: low, high, auto)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ImageSource {
    /// Remote URL.
    ///
    /// 远程 URL。
    Url { url: StaticRefStr },
    /// Base64 encoded data.
    ///
    /// Base64 编码数据。
    Base64 { data: StaticRefStr },
}

/// Unified Audio structure.
///
/// 统一的音频结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Audio {
    /// Base64 encoded audio data.
    ///
    /// Base64 编码的音频数据。
    pub data: StaticRefStr,
    /// Format (e.g., wav, mp3).
    ///
    /// 格式 (如 wav, mp3)。
    pub format: StaticRefStr,
}

impl Message {
    /// Create a new User message with text content.
    ///
    /// 创建带有文本内容的新 User 消息。
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text {
                text: text.into(),
                signature: None,
            }],
            name: None,
        }
    }

    /// Create a new Assistant message with text content.
    ///
    /// 创建带有文本内容的新 Assistant 消息。
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text {
                text: text.into(),
                signature: None,
            }],
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
    pub name: Option<StaticRefStr>,

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
    Text {
        index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<StaticRefStr>,
    },

    /// 推理/思维链内容增量 (对应 Claude Thinking Block)。
    Reasoning {
        index: u32,
        text: String,
        /// 可选：思维块的签名/验签数据。
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<StaticRefStr>,
    },

    /// 工具调用增量。
    ToolCall(DeltaToolCall),

    /// 拒绝内容增量 (OpenAI)。
    Refusal { refusal: StaticRefStr },
}

/// 增量工具调用。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaToolCall {
    /// 对应 Message 中 ToolCall 列表的索引。
    pub index: u32,

    /// 工具 ID (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<StaticRefStr>,

    /// 工具类型 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<StaticRefStr>,

    /// 函数信息增量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunction>,

    /// Thinking 签名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// 增量函数信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaFunction {
    /// 函数名 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,

    /// 参数片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<StaticRefStr>,
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
    Other(StaticRefStr),
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
    Start {
        role: Role,
        name: Option<StaticRefStr>,
    },

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
        id: Option<StaticRefStr>,
        #[serde(rename = "tool_type")]
        r#type: Option<StaticRefStr>,
        name: Option<StaticRefStr>,
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
                    Some(ContentPart::Text {
                        text: current,
                        signature: _,
                    }) => {
                        current.push_str(&text);
                    }
                    _ => {
                        content.push(ContentPart::Text {
                            text,
                            signature: None,
                        });
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
            name: name.map(Into::into),
        }
    }
}

/// Helper struct to assemble a complete Message from DeltaMessages.
///
/// 用于将多个 DeltaMessage 组装成完整 Message 的辅助结构。
#[derive(Debug, Clone, Default)]
pub struct MessageAssembler {
    role: Option<Role>,
    name: Option<StaticRefStr>,

    // Text and Reasoning parts: indexed
    content_parts: std::collections::BTreeMap<u32, AssembledPart>,

    // Tool calls: keyed by ID
    tool_calls: std::collections::HashMap<StaticRefStr, AssembledToolCall>,

    // Optimization: Map index to the current active tool ID
    // This allows O(1) lookup for incoming tool call deltas that lack an ID.
    active_tool_id: std::collections::HashMap<u32, StaticRefStr>,

    // Record appearance order: (index, id)
    tool_call_order: Vec<(u32, StaticRefStr)>,

    usage: Option<Usage>,
    finish_reason: Option<FinishReason>,
}

/// Assembled content part (Text and Reasoning only)
#[derive(Debug, Clone)]
enum AssembledPart {
    Text {
        text: String,
        signature: Option<StaticRefStr>,
    },
    Reasoning {
        text: String,
        signature: Option<StaticRefStr>,
    },
}

/// Assembled tool call
///
/// 已组装的工具调用
#[derive(Debug, Clone)]
struct AssembledToolCall {
    id: StaticRefStr,
    r#type: Option<StaticRefStr>,
    name: Option<StaticRefStr>,
    arguments: String,
    signature: Option<StaticRefStr>,
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
            if let Some(current) = self.usage.as_mut() {
                current.input_tokens = current.input_tokens.max(usage.input_tokens);
                current.output_tokens = current.output_tokens.max(usage.output_tokens);
                current.total_tokens = current.input_tokens + current.output_tokens;
            } else {
                self.usage = Some(usage);
            }
        }

        if let Some(content) = delta.content {
            for part in content {
                match part {
                    DeltaContentPart::Text {
                        index,
                        text,
                        signature,
                    } => {
                        let entry =
                            self.content_parts
                                .entry(index)
                                .or_insert(AssembledPart::Text {
                                    text: "".into(),
                                    signature: None,
                                });
                        if let AssembledPart::Text {
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
                    DeltaContentPart::Reasoning {
                        index,
                        text,
                        signature,
                    } => {
                        let entry =
                            self.content_parts
                                .entry(index)
                                .or_insert(AssembledPart::Reasoning {
                                    text: "".into(),
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
                        // 1. Determine Tool ID
                        let tool_id = if let Some(id) = tool_call.id {
                            // Explicit ID provided: update active mapping for this index
                            self.active_tool_id.insert(tool_call.index, id.clone());
                            id
                        } else {
                            // No ID: lookup active mapping for this index
                            // Fallback to generating a synthetic ID if not found (should be rare)
                            self.active_tool_id
                                .get(&tool_call.index)
                                .cloned()
                                .unwrap_or_else(|| format!("tool_{}", tool_call.index).into())
                        };

                        // 2. Get or create tool call entry
                        let entry = self.tool_calls.entry(tool_id.clone()).or_insert_with(|| {
                            // This is a NEW tool call (by ID). Record its order.
                            self.tool_call_order
                                .push((tool_call.index, tool_id.clone()));
                            AssembledToolCall {
                                id: tool_id.clone(),
                                r#type: None,
                                name: None,
                                arguments: "".into(),
                                signature: None,
                            }
                        });

                        // 3. Update fields
                        if let Some(tty) = tool_call.r#type {
                            entry.r#type = Some(tty);
                        }
                        if let Some(sig) = tool_call.signature {
                            entry.signature = Some(sig);
                        }
                        if let Some(func) = tool_call.function {
                            if let Some(fname) = func.name {
                                entry.name = Some(fname);
                            }
                            if let Some(fargs) = func.arguments {
                                entry.arguments.push_str(&fargs);
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
        let mut all_parts: Vec<(u32, ContentPart)> = Vec::new();

        // Add content parts
        for (index, part) in self.content_parts {
            let content_part = match part {
                AssembledPart::Text { text, signature } => ContentPart::Text { text, signature },
                AssembledPart::Reasoning { text, signature } => {
                    ContentPart::Reasoning { text, signature }
                }
            };
            all_parts.push((index, content_part));
        }

        // Add tool calls
        for (index, tool_id) in self.tool_call_order {
            if let Some(tool_call) = self.tool_calls.get(&tool_id) {
                let args_value: serde_json::Value = serde_json::from_str(&tool_call.arguments)
                    .unwrap_or(serde_json::Value::String(tool_call.arguments.to_string()));

                all_parts.push((
                    index,
                    ContentPart::ToolCall(crate::tool::ToolCall {
                        id: tool_call.id.clone(),
                        name: tool_call.name.clone().unwrap_or_default(),
                        arguments: args_value,
                        signature: tool_call.signature.clone(),
                    }),
                ));
            }
        }

        // Sort by index.
        // Important: Stable sort is preferred to keep relative order of items with same index (if any).
        all_parts.sort_by_key(|(index, _)| *index);

        let content = all_parts.into_iter().map(|(_, part)| part).collect();

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
    ///
    /// 根据索引获取特定的工具调用。
    pub fn get_tool_call(&self, index: u32) -> Option<crate::tool::ToolCall> {
        // Find the LAST tool ID associated with this index in the order list.
        let tool_id = self
            .tool_call_order
            .iter()
            .rev()
            .find(|(idx, _)| *idx == index)
            .map(|(_, id)| id)?;

        self.get_tool_call_by_id(tool_id)
    }

    /// Get a specific tool call by ID.
    ///
    /// 根据 ID 获取特定的工具调用。
    pub fn get_tool_call_by_id(&self, tool_id: &str) -> Option<crate::tool::ToolCall> {
        let tool_call = self.tool_calls.get(tool_id)?;
        let args_value: serde_json::Value = serde_json::from_str(&tool_call.arguments)
            .unwrap_or(serde_json::Value::String(tool_call.arguments.to_string()));

        Some(crate::tool::ToolCall {
            id: tool_call.id.clone(),
            name: tool_call.name.clone().unwrap_or_default(),
            arguments: args_value,
            signature: tool_call.signature.clone(),
        })
    }

    /// Get all tool call indices.
    ///
    /// 获取所有工具调用的索引。
    pub fn get_tool_call_indices(&self) -> Vec<u32> {
        self.tool_call_order.iter().map(|(idx, _)| *idx).collect()
    }
}

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
    emitted_tool_finishes: std::collections::HashSet<u32>,
    _marker: std::marker::PhantomData<E>,
}

impl<S, E> ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Send + 'static,
{
    pub fn into_boxed(
        self,
    ) -> ChatStream<futures::stream::BoxStream<'static, Result<DeltaMessage, E>>, E> {
        ChatStream {
            stream: self.stream.boxed(),
            assembler: self.assembler,
            pending_events: self.pending_events,
            stream_finished: self.stream_finished,
            start_event_emitted: self.start_event_emitted,
            finished_event_emitted: self.finished_event_emitted,
            emitted_tool_starts: self.emitted_tool_starts,
            emitted_tool_finishes: self.emitted_tool_finishes,
            _marker: std::marker::PhantomData,
        }
    }
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
            emitted_tool_finishes: std::collections::HashSet::new(),
            _marker: std::marker::PhantomData,
        }
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

                // Emit all completed tool calls that haven't been emitted yet
                let tool_indices = this.assembler.get_tool_call_indices();
                for index in tool_indices {
                    if !this.emitted_tool_finishes.contains(&index)
                        && let Some(tool_call) = this.assembler.get_tool_call(index)
                    {
                        this.pending_events
                            .push_back(ChatStreamEvent::ToolCallFinished(tool_call));
                        this.emitted_tool_finishes.insert(index);
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
                    let mut assembler_content = Vec::new();

                    if let Some(parts) = content {
                        for part in parts {
                            match part {
                                DeltaContentPart::Text {
                                    index,
                                    text,
                                    signature,
                                } => {
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Text {
                                            text: text.clone(),
                                        });
                                    }
                                    assembler_content.push(DeltaContentPart::Text {
                                        index,
                                        text,
                                        signature,
                                    });
                                }
                                DeltaContentPart::Reasoning {
                                    index,
                                    text,
                                    signature,
                                } => {
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Reasoning {
                                            text: text.clone(),
                                        });
                                    }
                                    assembler_content.push(DeltaContentPart::Reasoning {
                                        index,
                                        text,
                                        signature,
                                    });
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
                                    assembler_content.push(DeltaContentPart::ToolCall(tool));
                                }
                                other => {
                                    assembler_content.push(other);
                                }
                            }
                        }
                    }

                    // 4.4 Feed all content to Assembler (Text, Reasoning, ToolCalls)
                    if !assembler_content.is_empty() {
                        let content_delta = DeltaMessage {
                            role: None,
                            name: None,
                            content: Some(assembler_content),
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
