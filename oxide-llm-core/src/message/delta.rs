use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use crate::message::model::Role;

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

    /// Audio content delta.
    ///
    /// 音频内容增量。
    Audio {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<StaticRefStr>,
    },

    /// Image content delta.
    ///
    /// 图像内容增量。
    Image {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<StaticRefStr>,
    },

    /// Video content delta.
    ///
    /// 视频内容增量。
    Video {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<StaticRefStr>,
    },

    /// Document content delta.
    ///
    /// 文档内容增量。
    Document {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<StaticRefStr>,
    },
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

/// Token usage statistics for LLM requests and responses.
///
/// Token 使用量统计数据，统一抽象各类大模型（OpenAI, Claude, Gemini 等）的用量信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Usage {
    /// Total number of input tokens consumed by the prompt/messages.
    ///
    /// 提示词和消息所消耗的输入 Token 总数量。
    pub input_tokens: u32,

    /// Total number of output tokens generated by the model (inclusive total, including text, reasoning, and tool calls).
    ///
    /// 模型生成的输出 Token 总数量（权威计费总数，包含文本、推理/思维链及工具调用）。
    pub output_tokens: u32,

    /// Total token count for the request and response (input_tokens + output_tokens).
    ///
    /// 请求与响应消耗的总 Token 数量（input_tokens + output_tokens）。
    pub total_tokens: u32,

    /// Number of reasoning / thought tokens generated by the model during internal chain-of-thought processing.
    /// Note: This value is a sub-category breakdown included within `output_tokens` (`reasoning_tokens` <= `output_tokens`).
    ///
    /// 模型在内部思维链推理过程中生成的 Token 数量（如 Gemini total_thought_tokens, OpenAI reasoning_tokens, Claude thinking_tokens）。
    /// 注意：该数值包含在 `output_tokens` 总数之内，作为细分指标提供。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    /// Number of input tokens read from or used to create prompt cache.
    ///
    /// 从提示词缓存中读取或用于创建缓存的输入 Token 数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_input_tokens: Option<u32>,

    /// Number of output tokens read from cache.
    ///
    /// 从缓存中读取的输出 Token 数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_output_tokens: Option<u32>,

    /// Number of tokens spent on server-side or client-side tool execution.
    ///
    /// 在服务端或客户端工具执行中消耗的 Token 数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_tokens: Option<u32>,
}
