use super::{
    LogProbs,
    response::{OutputItem, OutputMessageContent, Response, ResponseError},
};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// OpenAI Response API 流式事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    /// 创建 Response 时触发。
    #[serde(rename = "response.created")]
    Created {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Response 处理中时触发。
    #[serde(rename = "response.in_progress")]
    InProgress {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// Response 入队时触发。
    #[serde(rename = "response.queued")]
    Queued {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 模型 Response 完成时触发。
    #[serde(rename = "response.completed")]
    Completed {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 模型 Response 失败时触发。
    #[serde(rename = "response.failed")]
    Failed {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 模型 Response 未完成时触发。
    #[serde(rename = "response.incomplete")]
    Incomplete {
        /// Response 对象状态。
        response: Response,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 处理过程中发生错误时触发。
    #[serde(rename = "response.error")]
    Error {
        /// Response 错误详细信息。
        error: ResponseError,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 添加新输出项时触发。
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 输出项对象。
        item: OutputItem,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 输出项完成时触发。
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 输出项对象。
        item: OutputItem,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 添加新内容部分时触发。
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 消息内容块对象。
        part: OutputMessageContent,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 内容部分完成时触发。
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 消息内容块对象。
        part: OutputMessageContent,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加文本 delta 时触发。
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 文本增量字符串。
        delta: String,

        /// 开启时的文本词元对数概率。
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<LogProbs>>,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 文本生成完成时触发。
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 完整的生成的文本字符串。
        text: String,

        /// 开启时的文本词元对数概率。
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<LogProbs>>,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加拒答 delta 时触发。
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 拒答文本增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 拒答文本完成时触发。
    #[serde(rename = "response.refusal.done")]
    RefusalDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 完整的拒答文本字符串。
        refusal: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加函数调用参数 delta 时触发。
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 调用的唯一 ID。
        #[serde(skip_serializing_if = "Option::is_none")]
        call_id: Option<StaticRefStr>,

        /// 函数参数增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 函数调用参数完成时触发。
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 调用的唯一 ID。
        #[serde(skip_serializing_if = "Option::is_none")]
        call_id: Option<StaticRefStr>,

        /// 函数名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,

        /// 完整的函数参数字符串。
        arguments: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加思考文本 delta 时触发。
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 思考文本增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 思考文本完成时触发。
    #[serde(rename = "response.reasoning_text.done")]
    ReasoningTextDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 完整的思考文本字符串。
        text: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加思考摘要 delta 时触发。
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 摘要部分在输出项中的索引。
        summary_index: u32,

        /// 思考摘要增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 思考摘要完成时触发。
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 摘要部分在输出项中的索引。
        summary_index: u32,

        /// 完整的思考摘要文本字符串。
        text: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加音频数据 delta 时触发。
    #[serde(rename = "response.audio.delta")]
    AudioDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 音频数据增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 音频数据完成时触发。
    #[serde(rename = "response.audio.done")]
    AudioDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 增加音频转写 delta 时触发。
    #[serde(rename = "response.audio_transcript.delta")]
    AudioTranscriptDelta {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 音频转写增量字符串。
        delta: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 音频转写完成时触发。
    #[serde(rename = "response.audio_transcript.done")]
    AudioTranscriptDone {
        /// 项目 ID。
        item_id: StaticRefStr,

        /// 输出项在响应列表中的索引。
        output_index: u32,

        /// 内容块在输出项中的索引。
        content_index: u32,

        /// 完整的音频转写文本字符串。
        transcript: String,

        /// 流式事件的序列号。
        #[serde(skip_serializing_if = "Option::is_none")]
        sequence_number: Option<u64>,
    },

    /// 未知或未导出的前向兼容事件。
    #[serde(other)]
    Unknown,
}
