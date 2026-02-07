use serde::{Deserialize, Serialize};

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
