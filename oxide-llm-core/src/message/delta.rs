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
