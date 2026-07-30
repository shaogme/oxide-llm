use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::request::MessagesRequest;
use super::response::MessagesResponse;

/// Request object for creating a Message Batch.
///
/// 创建 Message Batch 的请求对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    /// List of requests for prompt completion.
    ///
    /// 提示词补全请求列表。
    pub requests: Vec<BatchRequestItem>,
}

/// An individual request within a Message Batch.
///
/// Message Batch 中的单个请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestItem {
    /// Developer-provided ID created for each request in a Message Batch.
    ///
    /// 开发者为 Message Batch 中每个请求提供的唯一标识符。
    pub custom_id: StaticRefStr,

    /// Messages API creation parameters for the individual request.
    ///
    /// 单个请求的 Messages API 参数。
    pub params: MessagesRequest,
}

/// Status of a Message Batch.
///
/// Message Batch 的处理状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageBatchProcessingStatus {
    /// The batch is currently processing.
    ///
    /// 批处理正在处理中。
    InProgress,

    /// The batch is being canceled.
    ///
    /// 批处理正在取消中。
    Canceling,

    /// The batch processing has ended.
    ///
    /// 批处理已结束。
    Ended,
}

/// Request processing counts for a Message Batch.
///
/// Message Batch 的请求处理数量统计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatchRequestCounts {
    /// Number of requests currently processing.
    ///
    /// 正在处理的请求数量。
    pub processing: u32,

    /// Number of requests that succeeded.
    ///
    /// 成功完成的请求数量。
    pub succeeded: u32,

    /// Number of requests that errored.
    ///
    /// 发生错误的请求数量。
    pub errored: u32,

    /// Number of requests that were canceled.
    ///
    /// 被取消的请求数量。
    pub canceled: u32,

    /// Number of requests that expired.
    ///
    /// 已过期的请求数量。
    pub expired: u32,
}

/// Representation of a Message Batch.
///
/// Message Batch 对象结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// Unique identifier for the batch.
    ///
    /// 批处理的唯一标识符。
    pub id: String,

    /// Object type ("message_batch").
    ///
    /// 对象类型（固定为 "message_batch"）。
    #[serde(rename = "type")]
    pub r#type: String,

    /// Processing status of the batch.
    ///
    /// 批处理的处理状态。
    pub processing_status: MessageBatchProcessingStatus,

    /// Summary counts of requests in each status.
    ///
    /// 各状态下的请求数量统计。
    pub request_counts: MessageBatchRequestCounts,

    /// RFC 3339 datetime string of when processing ended.
    ///
    /// 处理结束的时间戳（RFC 3339 格式）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,

    /// RFC 3339 datetime string of when the batch was created.
    ///
    /// 批处理创建的时间戳（RFC 3339 格式）。
    pub created_at: String,

    /// RFC 3339 datetime string of when the batch will expire.
    ///
    /// 批处理过期的时间戳（RFC 3339 格式）。
    pub expires_at: String,

    /// RFC 3339 datetime string of when the batch was archived.
    ///
    /// 批处理归档的时间戳（RFC 3339 格式）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,

    /// RFC 3339 datetime string of when cancel was initiated.
    ///
    /// 发起取消的时间戳（RFC 3339 格式）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_initiated_at: Option<String>,

    /// URL to download the batch results.
    ///
    /// 下载批处理结果的 URL。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_url: Option<String>,
}

/// An individual line in the batch results JSONL file.
///
/// 批处理结果 JSONL 文件中的单行响应数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatchIndividualResponse {
    /// The custom ID provided in the batch request item.
    ///
    /// 批处理请求项中提供的自定义 ID。
    pub custom_id: String,

    /// Result of the individual request.
    ///
    /// 单个请求的处理结果。
    pub result: MessageBatchResult,
}

/// Result of an individual batch request.
///
/// 批处理中单个请求的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBatchResult {
    /// The request completed successfully.
    ///
    /// 请求成功完成。
    Succeeded {
        /// The generated message response.
        ///
        /// 生成的消息响应。
        message: Box<MessagesResponse>,
    },

    /// The request resulted in an error.
    ///
    /// 请求处理出错。
    Errored {
        /// Details of the error.
        ///
        /// 错误详情。
        error: BatchError,
    },

    /// The request was canceled.
    ///
    /// 请求已被取消。
    Canceled,

    /// The request expired before processing.
    ///
    /// 请求在处理前已过期。
    Expired,
}

/// Details of a batch error.
///
/// 批处理错误详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Type of the error.
    ///
    /// 错误类型。
    #[serde(rename = "type")]
    pub r#type: String,

    /// Human-readable error message.
    ///
    /// 可读的错误消息。
    pub message: String,
}

/// Response returned when a Message Batch is deleted.
///
/// 删除 Message Batch 时返回的响应对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedMessageBatch {
    /// ID of the deleted Message Batch.
    ///
    /// 已删除的 Message Batch 的 ID。
    pub id: String,

    /// Object type ("message_batch_deleted").
    ///
    /// 对象类型（固定为 "message_batch_deleted"）。
    #[serde(rename = "type")]
    pub r#type: String,
}
