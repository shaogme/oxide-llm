use super::response::{OutputItem, OutputMessageContent, Response, ResponseError};
use crate::openai::v1::LogProbs;
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// OpenAI Response API streaming event.
///
/// OpenAI Response API 流式事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    /// Emitted when a response is created.
    ///
    /// 创建 Response 时触发。
    #[serde(rename = "response.created")]
    Created {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a response is in progress.
    ///
    /// Response 处理中时触发。
    #[serde(rename = "response.in_progress")]
    InProgress {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a response is queued.
    ///
    /// Response 入队时触发。
    #[serde(rename = "response.queued")]
    Queued {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when the model response is complete.
    ///
    /// 模型 Response 完成时触发。
    #[serde(rename = "response.completed")]
    Completed {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when the model response fails.
    ///
    /// 模型 Response 失败时触发。
    #[serde(rename = "response.failed")]
    Failed {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when the model response is incomplete.
    ///
    /// 模型 Response 未完成时触发。
    #[serde(rename = "response.incomplete")]
    Incomplete {
        response: Response,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when an error occurs during processing.
    ///
    /// 处理过程中发生错误时触发。
    #[serde(rename = "response.error")]
    Error {
        error: ResponseError,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a new output item is added.
    ///
    /// 添加新输出项时触发。
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        output_index: u32,
        item: OutputItem,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when an output item is done.
    ///
    /// 输出项完成时触发。
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        output_index: u32,
        item: OutputItem,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a new content part is added.
    ///
    /// 添加新内容部分时触发。
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        part: OutputMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a content part is done.
    ///
    /// 内容部分完成时触发。
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        part: OutputMessageContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when there is an additional text delta.
    ///
    /// 增加文本 delta 时触发。
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<LogProbs>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when text generation is done.
    ///
    /// 文本生成完成时触发。
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<LogProbs>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when there is a refusal text delta.
    ///
    /// 增加拒答 delta 时触发。
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when refusal text is done.
    ///
    /// 拒答文本完成时触发。
    #[serde(rename = "response.refusal.done")]
    RefusalDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        refusal: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when there is a partial function-call arguments delta.
    ///
    /// 增加函数调用参数 delta 时触发。
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: StaticRefStr,
        output_index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        call_id: Option<StaticRefStr>,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when function-call arguments are finalized.
    ///
    /// 函数调用参数完成时触发。
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        item_id: StaticRefStr,
        output_index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        call_id: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
        arguments: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a delta is added to a reasoning text.
    ///
    /// 增加思考文本 delta 时触发。
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when reasoning text is completed.
    ///
    /// 思考文本完成时触发。
    #[serde(rename = "response.reasoning_text.done")]
    ReasoningTextDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a delta is added to a reasoning summary text.
    ///
    /// 增加思考摘要 delta 时触发。
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        item_id: StaticRefStr,
        output_index: u32,
        summary_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when a reasoning summary text is completed.
    ///
    /// 思考摘要完成时触发。
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        item_id: StaticRefStr,
        output_index: u32,
        summary_index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when there is an audio data delta.
    ///
    /// 增加音频数据 delta 时触发。
    #[serde(rename = "response.audio.delta")]
    AudioDelta {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when audio data is done.
    ///
    /// 音频数据完成时触发。
    #[serde(rename = "response.audio.done")]
    AudioDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when there is an audio transcript delta.
    ///
    /// 增加音频转写 delta 时触发。
    #[serde(rename = "response.audio_transcript.delta")]
    AudioTranscriptDelta {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Emitted when audio transcript is done.
    ///
    /// 音频转写完成时触发。
    #[serde(rename = "response.audio_transcript.done")]
    AudioTranscriptDone {
        item_id: StaticRefStr,
        output_index: u32,
        content_index: u32,
        transcript: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Catch-all for other or unknown events for forward compatibility.
    ///
    /// 未知或未导出的前向兼容事件。
    #[serde(other)]
    Unknown,
}
