use futures::{Stream, StreamExt};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
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

/// Image data source enum.
///
/// 图片数据源枚举。
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

    /// Create a new Message with specified role and content parts.
    ///
    /// 创建带有指定角色和内容部分的新 Message。
    pub fn new(role: Role, content: Vec<ContentPart>) -> Self {
        Self {
            role,
            content,
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

    /// Builder pattern helper to add a message to history.
    ///
    /// 链式调用添加一条消息到历史。
    pub fn with_message(mut self, message: Message) -> Self {
        self.add(message);
        self
    }
}

/// Delta message structure for streaming responses.
///
/// 增量消息结构，用于流式响应。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DeltaMessage {
    /// Role usually only appears in the first chunk, may be None afterwards.
    ///
    /// 角色通常只在第一个包出现，后续可能为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    /// Incremental message content parts.
    ///
    /// 消息内容的增量部分。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<DeltaContentPart>>,

    /// Sender name.
    ///
    /// 发送者名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,

    /// Finish reason (usually appears at the end of the stream).
    ///
    /// 结束原因 (通常在流的最后出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,

    /// Token usage statistics (may appear at start or end of stream).
    ///
    /// Token 使用情况 (可能在流的开始或结束出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Incremental content part.
///
/// 增量消息内容部分。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeltaContentPart {
    /// Text delta.
    ///
    /// 文本增量。
    Text {
        index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<StaticRefStr>,
    },

    /// Reasoning / Chain of Thought content delta (e.g. Claude Thinking Block).
    ///
    /// 推理/思维链内容增量 (对应 Claude Thinking Block)。
    Reasoning {
        index: u32,
        text: String,
        /// Optional signature or verification data for thinking block.
        ///
        /// 可选：思维块的签名/验签数据。
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<StaticRefStr>,
    },

    /// Tool call delta.
    ///
    /// 工具调用增量。
    ToolCall(DeltaToolCall),

    /// Refusal content delta (OpenAI specific).
    ///
    /// 拒绝内容增量 (OpenAI)。
    Refusal { refusal: StaticRefStr },
}

/// Incremental tool call structure.
///
/// 增量工具调用结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaToolCall {
    /// Index corresponding to ToolCall list in Message.
    ///
    /// 对应 Message 中 ToolCall 列表的索引。
    pub index: u32,

    /// Tool ID (usually only appears in the first chunk).
    ///
    /// 工具 ID (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<StaticRefStr>,

    /// Tool type (usually only appears in the first chunk).
    ///
    /// 工具类型 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<StaticRefStr>,

    /// Function information delta.
    ///
    /// 函数信息增量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunction>,

    /// Thinking signature.
    ///
    /// Thinking 签名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Incremental function information.
///
/// 增量函数信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaFunction {
    /// Function name (usually only appears in the first chunk).
    ///
    /// 函数名 (通常只在第一个包出现)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,

    /// Argument fragment.
    ///
    /// 参数片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<StaticRefStr>,
}

/// Finish reason for stream completion.
///
/// 结束原因。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Model stopped naturally.
    ///
    /// 模型自然停止生成。
    Stop,
    /// Reached maximum token limit.
    ///
    /// 达到最大 Token 限制。
    Length,
    /// Model requested tool calls.
    ///
    /// 模型请求调用工具。
    ToolCalls,
    /// Content intercepted by safety filter.
    ///
    /// 内容被安全过滤器拦截。
    ContentFilter,
    /// Other reasons.
    ///
    /// 其他原因。
    Other(StaticRefStr),
}

/// Token usage statistics.
///
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

    /// Reasoning block start.
    ///
    /// 思考/推理块开始。
    ReasoningStart,

    /// Reasoning stream chunk.
    ///
    /// 思考/推理流片段。
    Reasoning { text: String },

    /// Reasoning block end.
    ///
    /// 思考/推理块结束。
    ReasoningEnd,

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
    name: Option<StaticRefStr>,

    // Text and Reasoning parts: indexed
    content_parts: BTreeMap<u32, AssembledPart>,

    // Tool calls: keyed by ID
    tool_calls: HashMap<StaticRefStr, AssembledToolCall>,

    // Optimization: Map index to the current active tool ID
    // This allows O(1) lookup for incoming tool call deltas that lack an ID.
    active_tool_id: HashMap<u32, StaticRefStr>,

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

impl AssembledToolCall {
    fn to_tool_call(&self) -> crate::tool::ToolCall {
        let arguments: serde_json::Value = if self.arguments.trim().is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&self.arguments)
                .unwrap_or_else(|_| serde_json::Value::String(self.arguments.clone()))
        };

        crate::tool::ToolCall {
            id: self.id.clone(),
            name: self.name.clone().unwrap_or_default(),
            arguments,
            signature: self.signature.clone(),
        }
    }
}

impl MessageAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metadata fields (role, name, finish_reason, usage) from a DeltaMessage.
    ///
    /// 从 DeltaMessage 更新元数据字段 (role, name, finish_reason, usage)。
    pub fn add_metadata(&mut self, delta: &DeltaMessage) {
        if let Some(role) = delta.role {
            self.role = Some(role);
        }
        if let Some(name) = delta.name.as_ref() {
            self.name = Some(name.clone());
        }
        if let Some(reason) = delta.finish_reason.as_ref() {
            self.finish_reason = Some(reason.clone());
        }
        if let Some(usage) = delta.usage.as_ref() {
            if let Some(current) = self.usage.as_mut() {
                current.input_tokens = current.input_tokens.max(usage.input_tokens);
                current.output_tokens = current.output_tokens.max(usage.output_tokens);
                current.total_tokens = current.input_tokens + current.output_tokens;
            } else {
                self.usage = Some(usage.clone());
            }
        }
    }

    /// Add a single DeltaContentPart to the assembler.
    ///
    /// 添加单个 DeltaContentPart 到组装器。
    pub fn add_part(&mut self, part: DeltaContentPart) {
        match part {
            DeltaContentPart::Text {
                index,
                text,
                signature,
            } => {
                let entry = self
                    .content_parts
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
                let entry = self
                    .content_parts
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
                // Determine Tool ID and cache synthetic ID in active_tool_id for performance
                let tool_id = if let Some(id) = tool_call.id {
                    self.active_tool_id.insert(tool_call.index, id.clone());
                    id
                } else {
                    self.active_tool_id
                        .entry(tool_call.index)
                        .or_insert_with(|| format!("tool_{}", tool_call.index).into())
                        .clone()
                };

                let entry = self.tool_calls.entry(tool_id.clone()).or_insert_with(|| {
                    self.tool_call_order
                        .push((tool_call.index, tool_id.clone()));
                    AssembledToolCall {
                        id: tool_id,
                        r#type: None,
                        name: None,
                        arguments: "".into(),
                        signature: None,
                    }
                });

                if let Some(tty) = tool_call.r#type.filter(|t| !t.is_empty()) {
                    entry.r#type = Some(tty);
                }
                if let Some(sig) = tool_call.signature.filter(|s| !s.is_empty()) {
                    entry.signature = Some(sig);
                }
                if let Some(func) = tool_call.function {
                    if let Some(fname) = func.name.filter(|n| !n.is_empty()) {
                        entry.name = Some(fname);
                    }
                    if let Some(fargs) = func.arguments {
                        entry.arguments.push_str(&fargs);
                    }
                }
            }
            DeltaContentPart::Refusal { .. } => {}
        }
    }

    /// Add a delta message.
    ///
    /// 添加一个增量消息。
    pub fn add(&mut self, delta: DeltaMessage) {
        self.add_metadata(&delta);
        if let Some(content) = delta.content {
            for part in content {
                self.add_part(part);
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
                all_parts.push((index, ContentPart::ToolCall(tool_call.to_tool_call())));
            }
        }

        // Sort by index. Stable sort preserves relative order for items sharing the same index.
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
        Some(tool_call.to_tool_call())
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
    in_reasoning: bool,
    emitted_tool_starts: HashSet<u32>,
    emitted_tool_finishes: HashSet<u32>,
    _marker: PhantomData<E>,
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
            in_reasoning: self.in_reasoning,
            emitted_tool_starts: self.emitted_tool_starts,
            emitted_tool_finishes: self.emitted_tool_finishes,
            _marker: PhantomData,
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
            in_reasoning: false,
            emitted_tool_starts: HashSet::new(),
            emitted_tool_finishes: HashSet::new(),
            _marker: PhantomData,
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
                if this.in_reasoning {
                    this.in_reasoning = false;
                    this.pending_events.push_back(ChatStreamEvent::ReasoningEnd);
                    continue;
                }

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

                // Drain pending events before emitting Finished
                if let Some(event) = this.pending_events.pop_front() {
                    return Poll::Ready(Some(Ok(event)));
                }

                this.finished_event_emitted = true;

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
                    // 4.1 Extract content and update metadata directly
                    let content = delta.content.take();
                    this.assembler.add_metadata(&delta);

                    // 4.2 Emit Start Event if not yet emitted
                    if !this.start_event_emitted {
                        this.start_event_emitted = true;
                        this.pending_events.push_back(ChatStreamEvent::Start {
                            role: this.assembler.role.unwrap_or(Role::Assistant),
                            name: this.assembler.name.clone(),
                        });
                    }

                    // 4.3 Process Content and feed to Assembler directly
                    if let Some(parts) = content {
                        for part in parts {
                            match &part {
                                DeltaContentPart::Text { text, .. } => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Text {
                                            text: text.clone(),
                                        });
                                    }
                                }
                                DeltaContentPart::Reasoning { text, .. } => {
                                    if !this.in_reasoning {
                                        this.in_reasoning = true;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningStart);
                                    }
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Reasoning {
                                            text: text.clone(),
                                        });
                                    }
                                }
                                DeltaContentPart::ToolCall(tool) => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
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
                                }
                                DeltaContentPart::Refusal { .. } => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
                                }
                            }
                            this.assembler.add_part(part);
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{executor::block_on, stream};

    #[test]
    fn test_message_and_history_builder() {
        let msg1 = Message::user("hello");
        assert_eq!(msg1.role, Role::User);
        assert_eq!(
            msg1.content,
            vec![ContentPart::Text {
                text: "hello".into(),
                signature: None,
            }]
        );

        let msg2 = Message::assistant("world");
        let history = MessageHistory::new()
            .with_message(msg1)
            .with_message(msg2);
        assert_eq!(history.messages.len(), 2);
    }

    #[test]
    fn test_message_assembler_interleaved_tool_calls() {
        let mut assembler = MessageAssembler::new();

        assembler.add(DeltaMessage {
            role: Some(Role::Assistant),
            content: Some(vec![
                DeltaContentPart::Text {
                    index: 0,
                    text: "Let me search for that.".into(),
                    signature: None,
                },
                DeltaContentPart::ToolCall(DeltaToolCall {
                    index: 1,
                    id: Some("call_abc123".into()),
                    r#type: Some("function".into()),
                    function: Some(DeltaFunction {
                        name: Some("search".into()),
                        arguments: Some("{\"query\":".into()),
                    }),
                    signature: None,
                }),
            ]),
            ..Default::default()
        });

        // Fragment without tool ID, relying on index -> synthetic/cached tool ID lookup
        assembler.add(DeltaMessage {
            content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                index: 1,
                id: None,
                r#type: None,
                function: Some(DeltaFunction {
                    name: None,
                    arguments: Some("\"rust\"}".into()),
                }),
                signature: None,
            })]),
            finish_reason: Some(FinishReason::ToolCalls),
            usage: Some(Usage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            }),
            ..Default::default()
        });

        assert_eq!(assembler.finish_reason(), Some(FinishReason::ToolCalls));
        assert_eq!(
            assembler.usage(),
            Some(Usage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            })
        );

        let assembled_tool = assembler.get_tool_call(1).unwrap();
        assert_eq!(assembled_tool.id, "call_abc123");
        assert_eq!(assembled_tool.name, "search");
        assert_eq!(
            assembled_tool.arguments,
            serde_json::json!({ "query": "rust" })
        );

        let final_msg = assembler.build();
        assert_eq!(final_msg.role, Role::Assistant);
        assert_eq!(final_msg.content.len(), 2);
        assert_eq!(
            final_msg.content[0],
            ContentPart::Text {
                text: "Let me search for that.".into(),
                signature: None,
            }
        );
        assert_eq!(final_msg.content[1], ContentPart::ToolCall(assembled_tool));
    }

    #[test]
    fn test_chat_stream_tool_call_events() {
        block_on(async {
            let deltas = vec![
                Ok::<_, String>(DeltaMessage {
                    role: Some(Role::Assistant),
                    content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                        index: 0,
                        id: Some("call_xyz".into()),
                        r#type: Some("function".into()),
                        function: Some(DeltaFunction {
                            name: Some("calculator".into()),
                            arguments: Some("{\"expr\":".into()),
                        }),
                        signature: None,
                    })]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                        index: 0,
                        id: None,
                        r#type: None,
                        function: Some(DeltaFunction {
                            name: None,
                            arguments: Some("\"1+1\"}".into()),
                        }),
                        signature: None,
                    })]),
                    finish_reason: Some(FinishReason::ToolCalls),
                    ..Default::default()
                }),
            ];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ToolCallStart {
                        index: 0,
                        id: Some("call_xyz".into()),
                        r#type: Some("function".into()),
                        name: Some("calculator".into()),
                    },
                    ChatStreamEvent::ToolCallFinished(crate::tool::ToolCall {
                        id: "call_xyz".into(),
                        name: "calculator".into(),
                        arguments: serde_json::json!({ "expr": "1+1" }),
                        signature: None,
                    }),
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: Some(FinishReason::ToolCalls),
                    }
                ]
            );
        });
    }

    #[test]
    fn test_chat_stream_reasoning_lifecycle() {
        block_on(async {
            let deltas = vec![
                Ok::<_, String>(DeltaMessage {
                    role: Some(Role::Assistant),
                    content: Some(vec![DeltaContentPart::Reasoning {
                        index: 0,
                        text: "thinking part 1".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::Reasoning {
                        index: 0,
                        text: "thinking part 2".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::Text {
                        index: 1,
                        text: "final answer".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
            ];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ReasoningStart,
                    ChatStreamEvent::Reasoning {
                        text: "thinking part 1".to_string()
                    },
                    ChatStreamEvent::Reasoning {
                        text: "thinking part 2".to_string()
                    },
                    ChatStreamEvent::ReasoningEnd,
                    ChatStreamEvent::Text {
                        text: "final answer".to_string()
                    },
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: None,
                    }
                ]
            );
        });
    }

    #[test]
    fn test_chat_stream_reasoning_end_on_stream_finish() {
        block_on(async {
            let deltas = vec![Ok::<_, String>(DeltaMessage {
                role: Some(Role::Assistant),
                content: Some(vec![DeltaContentPart::Reasoning {
                    index: 0,
                    text: "only reasoning".to_string(),
                    signature: None,
                }]),
                ..Default::default()
            })];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ReasoningStart,
                    ChatStreamEvent::Reasoning {
                        text: "only reasoning".to_string()
                    },
                    ChatStreamEvent::ReasoningEnd,
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: None,
                    }
                ]
            );
        });
    }
}
