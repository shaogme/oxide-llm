use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::{
    response::{Interaction, InteractionStatus, Usage},
    step::Step,
};

/// Server-Sent Event (SSE) for streamed interactions.
///
/// 流式 Interaction 的服务器发送事件（SSE）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum InteractionSseEvent {
    /// Interaction created event.
    ///
    /// Interaction 创建事件。
    #[serde(rename = "interaction.created")]
    InteractionCreated(InteractionCreatedEvent),
    /// Interaction completed event.
    ///
    /// Interaction 完成事件。
    #[serde(rename = "interaction.completed")]
    InteractionCompleted(InteractionCompletedEvent),
    /// Interaction status update event.
    ///
    /// Interaction 状态更新事件。
    #[serde(rename = "interaction.status_update")]
    InteractionStatusUpdate(InteractionStatusUpdate),
    /// Step start event.
    ///
    /// 步骤开始事件。
    #[serde(rename = "step.start")]
    StepStart(StepStart),
    /// Step delta event.
    ///
    /// 步骤增量事件。
    #[serde(rename = "step.delta")]
    StepDelta(StepDelta),
    /// Step stop event.
    ///
    /// 步骤停止事件。
    #[serde(rename = "step.stop")]
    StepStop(StepStop),
    /// Stream error event.
    ///
    /// 流错误事件。
    #[serde(rename = "error")]
    Error(ErrorEvent),
}

/// Interaction created event payload.
///
/// Interaction 创建事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCreatedEvent {
    /// Interaction resource.
    ///
    /// Interaction 资源。
    pub interaction: Interaction,
    /// Event ID to resume stream.
    ///
    /// 恢复流的事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Interaction completed event payload.
///
/// Interaction 完成事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCompletedEvent {
    /// Interaction resource.
    ///
    /// Interaction 资源。
    pub interaction: Interaction,
    /// Event ID to resume stream.
    ///
    /// 恢复流的事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Interaction status update payload.
///
/// Interaction 状态更新载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionStatusUpdate {
    /// Interaction ID.
    ///
    /// Interaction ID。
    pub interaction_id: String,
    /// Updated status.
    ///
    /// 更新的状态。
    pub status: InteractionStatus,
    /// Event ID.
    ///
    /// 事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Step start event payload.
///
/// 步骤开始事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStart {
    /// Step index.
    ///
    /// 步骤索引。
    pub index: i32,
    /// Step details.
    ///
    /// 步骤详情。
    pub step: Step,
    /// Event ID.
    ///
    /// 事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Step delta event payload.
///
/// 步骤增量事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDelta {
    /// Step index.
    ///
    /// 步骤索引。
    pub index: i32,
    /// Delta data payload.
    ///
    /// 增量数据载荷。
    pub delta: StepDeltaData,
    /// Event ID.
    ///
    /// 事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Step stop event payload.
///
/// 步骤停止事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStop {
    /// Step index.
    ///
    /// 步骤索引。
    pub index: i32,
    /// Step usage statistics.
    ///
    /// 步骤的使用量统计。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_usage: Option<Usage>,
    /// Cumulative session usage statistics.
    ///
    /// 累计的会话使用量统计。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Event ID.
    ///
    /// 事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Error event payload.
///
/// 错误事件载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error details.
    ///
    /// 错误细节。
    pub error: StreamError,
    /// Event ID.
    ///
    /// 事件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// Error payload in stream.
///
/// 流中的错误载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamError {
    /// Error code string or status code.
    ///
    /// 错误代码字符串或状态码。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human readable error message.
    ///
    /// 人类可读的错误消息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Step delta data variants.
///
/// 步骤增量数据变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepDeltaData {
    /// Text delta.
    ///
    /// 文本增量。
    #[serde(rename = "text")]
    Text(TextDelta),
    /// Thought summary delta.
    ///
    /// 思考摘要增量。
    #[serde(rename = "thought_summary")]
    ThoughtSummary(ThoughtSummaryDelta),
    /// Thought signature delta.
    ///
    /// 思考签名增量。
    #[serde(rename = "thought_signature")]
    ThoughtSignature(ThoughtSignatureDelta),
    /// Audio delta.
    ///
    /// 音频增量。
    #[serde(rename = "audio")]
    Audio(AudioDelta),
    /// Image delta.
    ///
    /// 图像增量。
    #[serde(rename = "image")]
    Image(ImageDelta),
    /// Video delta.
    ///
    /// 视频增量。
    #[serde(rename = "video")]
    Video(VideoDelta),
    /// Document delta.
    ///
    /// 文档增量。
    #[serde(rename = "document")]
    Document(DocumentDelta),
    /// Arguments delta.
    ///
    /// 参数增量。
    #[serde(rename = "arguments")]
    Arguments(ArgumentsDelta),
}

/// Text delta.
///
/// 文本增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDelta {
    /// Text snippet.
    ///
    /// 文本片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Character index.
    ///
    /// 字符索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
}

/// Thought summary delta.
///
/// 思考摘要增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtSummaryDelta {
    /// Thought summary text.
    ///
    /// 思考摘要文本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Index.
    ///
    /// 索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
}

/// Thought signature delta.
///
/// 思考签名增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtSignatureDelta {
    /// Signature string.
    ///
    /// 签名字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Audio delta.
///
/// 音频增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDelta {
    /// Base64 audio bytes chunk.
    ///
    /// Base64 音频字节块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// Mime type.
    ///
    /// MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
}

/// Image delta.
///
/// 图像增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDelta {
    /// Base64 image bytes chunk.
    ///
    /// Base64 图像字节块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// Mime type.
    ///
    /// MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// URI.
    ///
    /// URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// Video delta.
///
/// 视频增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDelta {
    /// Base64 video bytes chunk.
    ///
    /// Base64 视频字节块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// Mime type.
    ///
    /// MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// URI.
    ///
    /// URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// Document delta.
///
/// 文档增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDelta {
    /// Base64 document bytes chunk.
    ///
    /// Base64 文档字节块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// Mime type.
    ///
    /// MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// URI.
    ///
    /// URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// Arguments delta.
///
/// 参数增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentsDelta {
    /// Arguments string snippet.
    ///
    /// 参数字符串片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
