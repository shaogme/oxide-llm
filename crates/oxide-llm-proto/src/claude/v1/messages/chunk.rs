use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::response::{MessagesResponse, StopReason};
use super::{
    BashCodeExecutionToolResultBlock, Citation, CodeExecutionToolResultBlock, ContainerUploadBlock,
    ImageBlock, RedactedThinkingBlock, SearchResultBlock, ServerToolUseBlock, TextBlock,
    TextEditorCodeExecutionToolResultBlock, ToolSearchToolResultBlock, ToolUseBlock,
    WebFetchToolResultBlock, WebSearchToolResultBlock,
};

/// Server-Sent Events (SSE) stream event payload in Messages API streaming response.
///
/// Messages API 流式响应中的 SSE 事件负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageStreamEvent {
    /// Initial event sent at the start of a streamed message response.
    ///
    /// 流式消息响应开始时的初始事件。
    MessageStart {
        /// Initial message state object.
        ///
        /// 初始消息状态对象。
        message: MessagesResponse,
    },

    /// Event sent when a new content block begins.
    ///
    /// 新内容块开始生成时的事件。
    ContentBlockStart {
        /// 0-based index of the content block in the message content array.
        ///
        /// 内容块在消息内容数组中的 0 纪元索引。
        index: u32,

        /// Initial content block.
        ///
        /// 初始内容块。
        content_block: ChunkContentBlock,
    },

    /// Ping event sent periodically to keep the connection alive.
    ///
    /// 定期发送以保持连接活性的 Ping 事件。
    Ping,

    /// Incremental delta update for an active content block.
    ///
    /// 活动内容块的增量 Delta 更新事件。
    ContentBlockDelta {
        /// Index of the content block being updated.
        ///
        /// 正在更新的内容块索引。
        index: u32,

        /// Incremental delta content.
        ///
        /// 增量 Delta 内容。
        delta: ChunkContentBlockDelta,
    },

    /// Event sent when a content block reaches completion.
    ///
    /// 内容块生成完成时的事件。
    ContentBlockStop {
        /// Index of the stopped content block.
        ///
        /// 已停止的内容块索引。
        index: u32,
    },

    /// Delta update for top-level message fields.
    ///
    /// 顶层消息字段的 Delta 更新事件。
    MessageDelta {
        /// Delta changes for message attributes.
        ///
        /// 消息属性的增量变化。
        delta: ChunkMessageDelta,

        /// Usage statistics delta.
        ///
        /// Usage 使用情况的增量。
        usage: ChunkMessageDeltaUsage,
    },

    /// Event sent when the entire message stream ends.
    ///
    /// 整个消息流结束时的事件。
    MessageStop,

    /// Error event sent if an unrecoverable error occurs during streaming.
    ///
    /// 流式生成过程中出现不可恢复错误时的 Error 事件。
    Error {
        /// Error payload details.
        ///
        /// 错误负载详情。
        error: ChunkError,
    },
}

/// Content block variant in a `content_block_start` stream event.
///
/// `content_block_start` 流事件中的内容块变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkContentBlock {
    /// Text block.
    ///
    /// 文本块。
    Text(TextBlock),

    /// Image block.
    ///
    /// 图像块。
    Image(ImageBlock),

    /// Tool use block.
    ///
    /// 工具调用块。
    ToolUse(ToolUseBlock),

    /// Thinking block.
    ///
    /// 思考块。
    Thinking(ChunkThinkingBlock),

    /// Redacted thinking block.
    ///
    /// 脱敏思考块。
    RedactedThinking(RedactedThinkingBlock),

    /// Search result block.
    ///
    /// 搜索结果块。
    #[serde(rename = "search_result")]
    SearchResult(SearchResultBlock),

    /// Server tool use block.
    ///
    /// 服务端工具调用块。
    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlock),

    /// Web search tool result block.
    ///
    /// Web 搜索工具结果块。
    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlock),

    /// Web fetch tool result block.
    ///
    /// Web 获取工具结果块。
    #[serde(rename = "web_fetch_tool_result")]
    WebFetchToolResult(WebFetchToolResultBlock),

    /// Code execution tool result block.
    ///
    /// 代码执行工具结果块。
    #[serde(rename = "code_execution_tool_result")]
    CodeExecutionToolResult(CodeExecutionToolResultBlock),

    /// Bash code execution tool result block.
    ///
    /// Bash 代码执行工具结果块。
    #[serde(rename = "bash_code_execution_tool_result")]
    BashCodeExecutionToolResult(BashCodeExecutionToolResultBlock),

    /// Text editor code execution tool result block.
    ///
    /// 文本编辑器代码执行工具结果块。
    #[serde(rename = "text_editor_code_execution_tool_result")]
    TextEditorCodeExecutionToolResult(TextEditorCodeExecutionToolResultBlock),

    /// Tool search tool result block.
    ///
    /// 工具搜索工具结果块。
    #[serde(rename = "tool_search_tool_result")]
    ToolSearchToolResult(ToolSearchToolResultBlock),

    /// Container upload block.
    ///
    /// 容器上传文件内容块。
    #[serde(rename = "container_upload")]
    ContainerUpload(ContainerUploadBlock),
}

/// Initial state of a thinking content block in streaming mode.
///
/// 流式模式下 Thinking 块的初始状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkThinkingBlock {
    /// Thinking content text.
    ///
    /// 思考文本内容。
    pub thinking: String,
}

/// Incremental delta payload in a `content_block_delta` event.
///
/// `content_block_delta` 事件中的增量 Delta 负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkContentBlockDelta {
    /// Text content delta snippet.
    ///
    /// 文本内容的增量片段。
    TextDelta {
        /// Text snippet.
        ///
        /// 文本片段。
        text: String,
    },

    /// Input JSON delta snippet for tool use parameter stream.
    ///
    /// 工具调用参数流的 JSON 增量片段。
    InputJsonDelta {
        /// Partial JSON string.
        ///
        /// 局部 JSON 字符串。
        partial_json: StaticRefStr,
    },

    /// Thinking content delta snippet.
    ///
    /// 思考内容的增量片段。
    ThinkingDelta {
        /// Thinking snippet.
        ///
        /// 思考片段。
        thinking: String,
    },

    /// Signature delta for thinking integrity.
    ///
    /// 思考完整性签名的 Delta。
    SignatureDelta {
        /// Signature string.
        ///
        /// 签名字符串。
        signature: StaticRefStr,
    },

    /// Citation delta event.
    ///
    /// 引用增量事件。
    CitationsDelta {
        /// Citation details.
        ///
        /// 引用详情。
        citation: Citation,
    },
}

/// Top-level message field changes in `message_delta` event.
///
/// `message_delta` 事件中的顶层消息字段变化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessageDelta {
    /// Final stop reason if the message stopped during this event.
    ///
    /// 消息在该事件中停止时的停止原因。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    /// Custom stop sequence generated.
    ///
    /// 生成的自定义停止序列。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<StaticRefStr>,
}

/// Usage statistics delta in `message_delta` event.
///
/// `message_delta` 事件中的 Usage 使用情况增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessageDeltaUsage {
    /// Optional input tokens count.
    ///
    /// 可选的输入 token 数量。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,

    /// Output tokens count generated in this turn.
    ///
    /// 本次生成的输出 token 数量。
    pub output_tokens: u32,
}

/// Error details in an `error` stream event.
///
/// `error` 流事件中的错误详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkError {
    /// Error type tag.
    ///
    /// 错误类型。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Human-readable error message.
    ///
    /// 可读的错误消息。
    pub message: StaticRefStr,
}
