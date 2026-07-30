use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::caller::Caller;
use super::citation::{Citation, CitationsConfig};
use super::thinking::{RedactedThinkingBlock, ThinkingBlock};

/// Role of a message in a conversation.
///
/// 对话中消息的角色。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// User role.
    ///
    /// 用户角色。
    User,

    /// Assistant role.
    ///
    /// 助手角色。
    Assistant,
}

/// Message content, which can be either a text string or a list of content blocks.
///
/// 消息内容，可以是纯文本字符串或内容块列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Plain text string content.
    ///
    /// 纯文本内容。
    Text(StaticRefStr),

    /// Structured list of content blocks.
    ///
    /// 结构化的内容块列表。
    Blocks(Vec<ContentBlock>),
}

/// Content block generated or sent in a conversation.
///
/// 对话中生成或发送的内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content block.
    ///
    /// 文本内容块。
    Text(TextBlock),

    /// Image content block.
    ///
    /// 图像内容块。
    Image(ImageBlock),

    /// Tool invocation requested by the model.
    ///
    /// 模型请求的工具调用块。
    ToolUse(ToolUseBlock),

    /// Result of a tool invocation.
    ///
    /// 工具调用的结果块。
    ToolResult(ToolResultBlock),

    /// Document content block.
    ///
    /// 文档内容块。
    Document(DocumentBlock),

    /// Reasoning content block from extended thinking.
    ///
    /// 深度思考生成的推理内容块。
    Thinking(ThinkingBlock),

    /// Redacted thinking content block.
    ///
    /// 已脱敏的思考内容块。
    RedactedThinking(RedactedThinkingBlock),

    /// Search result content block.
    ///
    /// 搜索结果内容块。
    #[serde(rename = "search_result")]
    SearchResult(SearchResultBlock),

    /// Server-side tool use content block.
    ///
    /// 服务端工具调用块。
    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlock),

    /// Web search tool result content block.
    ///
    /// Web 搜索工具结果块。
    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlock),

    /// Web fetch tool result content block.
    ///
    /// Web 获取工具结果块。
    #[serde(rename = "web_fetch_tool_result")]
    WebFetchToolResult(WebFetchToolResultBlock),

    /// Code execution tool result content block.
    ///
    /// 代码执行工具结果块。
    #[serde(rename = "code_execution_tool_result")]
    CodeExecutionToolResult(CodeExecutionToolResultBlock),

    /// Bash code execution tool result content block.
    ///
    /// Bash 代码执行工具结果块。
    #[serde(rename = "bash_code_execution_tool_result")]
    BashCodeExecutionToolResult(BashCodeExecutionToolResultBlock),

    /// Text editor code execution tool result content block.
    ///
    /// 文本编辑器代码执行工具结果块。
    #[serde(rename = "text_editor_code_execution_tool_result")]
    TextEditorCodeExecutionToolResult(TextEditorCodeExecutionToolResultBlock),

    /// Tool search tool result content block.
    ///
    /// 工具搜索工具结果块。
    #[serde(rename = "tool_search_tool_result")]
    ToolSearchToolResult(ToolSearchToolResultBlock),

    /// Container upload content block.
    ///
    /// 容器上传文件内容块。
    #[serde(rename = "container_upload")]
    ContainerUpload(ContainerUploadBlock),
}

/// Text content block structure.
///
/// 文本内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// The text content.
    ///
    /// 文本内容。
    pub text: String,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,

    /// Citations supporting this text block.
    ///
    /// 支持此文本块的引用来源。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Citation>>,
}

/// Image content block structure.
///
/// 图像内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    /// Image source data.
    ///
    /// 图像数据源。
    pub source: ImageSource,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Source specification for an image.
///
/// 图像的数据源规格。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    /// Base64-encoded image data.
    ///
    /// Base64 编码的图像数据。
    Base64 {
        /// Source type ("base64").
        ///
        /// 数据源类型（固定为 "base64"）。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// MIME type of the image.
        ///
        /// 图像的 MIME 类型。
        media_type: StaticRefStr,

        /// Raw Base64 string data.
        ///
        /// Raw Base64 编码数据。
        data: StaticRefStr,
    },

    /// Image specified by URL.
    ///
    /// 通过 URL 指定的图像。
    Url {
        /// Source type ("url").
        ///
        /// 数据源类型（固定为 "url"）。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// Image URL.
        ///
        /// 图像 URL。
        url: StaticRefStr,
    },
}

/// Tool use content block structure.
///
/// 工具调用内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// Unique identifier for this tool use invocation.
    ///
    /// 此工具调用的唯一标识符。
    pub id: StaticRefStr,

    /// Name of the tool being called.
    ///
    /// 被调用的工具名称。
    pub name: StaticRefStr,

    /// Input parameters passed to the tool.
    ///
    /// 传递给工具的输入参数。
    pub input: Value,

    /// Caller information.
    ///
    /// 调用方信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Caller>,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Tool result content block structure.
///
/// 工具执行结果内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    /// ID of the tool use invocation this result responds to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Output content of the tool execution.
    ///
    /// 工具执行输出的内容。
    pub content: ToolResultContent,

    /// Whether the tool execution resulted in an error.
    ///
    /// 工具执行过程是否出错。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Content payload for a tool result.
///
/// 工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Plain text result content.
    ///
    /// 纯文本结果。
    Text(String),

    /// Structured list of content blocks.
    ///
    /// 结构化的内容块列表。
    Blocks(Vec<ContentBlock>),
}

/// Document content block structure.
///
/// 文档内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBlock {
    /// Document source data.
    ///
    /// 文档数据源。
    pub source: DocumentSource,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,

    /// Optional document title.
    ///
    /// 可选的文档标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StaticRefStr>,

    /// Optional document context background.
    ///
    /// 可选的文档背景说明信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<StaticRefStr>,

    /// Citation configurations for this document.
    ///
    /// 此文档的引用配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationsConfig>,
}

/// Source specification for a document.
///
/// 文档的数据源规格。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentSource {
    /// Base64-encoded PDF document data.
    ///
    /// Base64 编码的 PDF 文档数据。
    Base64 {
        /// Source type ("base64").
        ///
        /// 数据源类型。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// Media type ("application/pdf").
        ///
        /// 媒体类型。
        media_type: StaticRefStr,

        /// Base64 encoded string data.
        ///
        /// Base64 编码数据。
        data: StaticRefStr,
    },

    /// Document specified by URL.
    ///
    /// 通过 URL 指定的文档。
    Url {
        /// Source type ("url").
        ///
        /// 数据源类型。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// Document URL.
        ///
        /// 文档 URL。
        url: StaticRefStr,
    },

    /// Plain text document content.
    ///
    /// 纯文本文档内容。
    Text {
        /// Source type ("text").
        ///
        /// 数据源类型。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// Media type ("text/plain").
        ///
        /// 媒体类型。
        media_type: StaticRefStr,

        /// Text data.
        ///
        /// 文本内容。
        data: StaticRefStr,
    },

    /// Embedded content block payload.
    ///
    /// 嵌套的 Content 负载。
    Content {
        /// Source type ("content").
        ///
        /// 数据源类型。
        #[serde(rename = "type")]
        r#type: StaticRefStr,

        /// Content payload.
        ///
        /// 内容负载。
        content: Content,
    },
}

/// Cache control specification for prompt caching.
///
/// 提示词缓存控制说明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Cache control type ("ephemeral").
    ///
    /// 缓存类型（固定为 "ephemeral"）。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Time-to-live for the cache breakpoint ("5m" or "1h").
    ///
    /// 缓存生存时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<StaticRefStr>,
}

/// Search result block structure.
///
/// 搜索结果内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultBlock {
    /// Text blocks contained in the search result.
    ///
    /// 搜索结果包含的文本块列表。
    pub content: Vec<TextBlock>,

    /// Source of the search result.
    ///
    /// 搜索结果的来源。
    pub source: StaticRefStr,

    /// Title of the search result.
    ///
    /// 搜索结果的标题。
    pub title: StaticRefStr,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,

    /// Citation configurations for this search result.
    ///
    /// 引用配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationsConfig>,
}

/// Server tool use block structure.
///
/// 服务端工具调用内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlock {
    /// Unique tool use ID.
    ///
    /// 工具调用的唯一 ID。
    pub id: StaticRefStr,

    /// Name of the server-side tool (e.g. "web_search", "web_fetch").
    ///
    /// 服务端工具名称。
    pub name: StaticRefStr,

    /// Input parameters.
    ///
    /// 输入参数。
    pub input: Value,

    /// Caller information.
    ///
    /// 调用方信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Caller>,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Web search tool result block structure.
///
/// Web 搜索工具结果块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Content array of web search results.
    ///
    /// Web 搜索结果列表。
    pub content: Vec<WebSearchResultItem>,

    /// Caller information.
    ///
    /// 调用方信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Caller>,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Item inside a web search tool result block.
///
/// Web 搜索工具结果列表中的条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSearchResultItem {
    /// Successful web search result.
    ///
    /// 成功的 Web 搜索结果。
    WebSearchResult(WebSearchResult),

    /// Error during web search execution.
    ///
    /// Web 搜索执行过程中的错误。
    WebSearchToolResultError(WebSearchToolResultError),
}

/// Individual web search result item.
///
/// 单个 Web 搜索结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    /// Encrypted snippet content.
    ///
    /// 加密的摘要内容。
    pub encrypted_content: StaticRefStr,

    /// Web page title.
    ///
    /// 网页标题。
    pub title: StaticRefStr,

    /// Web page URL.
    ///
    /// 网页 URL。
    pub url: StaticRefStr,

    /// Page age metadata.
    ///
    /// 网页发布/更新时间元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_age: Option<StaticRefStr>,
}

/// Error details for web search tool execution.
///
/// Web 搜索工具执行出错时的信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultError {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,
}

/// Web fetch tool result block structure.
///
/// Web 获取工具结果块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Content of the web fetch tool result.
    ///
    /// Web 获取工具结果的内容。
    pub content: WebFetchToolResultContent,

    /// Caller information.
    ///
    /// 调用方信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Caller>,

    /// Cache control breakpoint configuration.
    ///
    /// 缓存控制断点配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Content payload for a Web Fetch tool result.
///
/// Web 获取工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebFetchToolResultContent {
    /// Web fetch error block.
    ///
    /// Web 获取错误块。
    Error(WebFetchToolResultErrorBlock),

    /// Web fetch successful result block.
    ///
    /// Web 获取成功结果块。
    Result(WebFetchBlock),
}

/// Error block for web fetch tool.
///
/// Web 获取工具错误块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultErrorBlock {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,
}

/// Web fetch successful block.
///
/// Web 获取成功块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchBlock {
    /// Document content fetched.
    ///
    /// 获取到的文档内容。
    pub content: DocumentBlock,

    /// ISO 8601 timestamp string when content was retrieved.
    ///
    /// 内容获取的时间戳。
    pub retrieved_at: StaticRefStr,

    /// URL fetched.
    ///
    /// 获取的 URL。
    pub url: StaticRefStr,
}

/// Code execution tool result block.
///
/// 代码执行工具结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Execution output content payload.
    ///
    /// 执行输出的内容负载。
    pub content: CodeExecutionToolResultContent,
}

/// Content payload for code execution tool result.
///
/// 代码执行工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeExecutionToolResultContent {
    /// Error during execution.
    ///
    /// 执行过程中的错误。
    Error(CodeExecutionToolResultError),

    /// Standard code execution result.
    ///
    /// 标准代码执行结果。
    Result(CodeExecutionResultBlock),

    /// Encrypted code execution result.
    ///
    /// 加密代码执行结果。
    EncryptedResult(EncryptedCodeExecutionResultBlock),
}

/// Error block for code execution.
///
/// 代码执行工具错误块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionToolResultError {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,
}

/// Standard code execution result block.
///
/// 标准代码执行结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionResultBlock {
    /// Generated output file blocks.
    ///
    /// 生成的输出文件块。
    pub content: Vec<CodeExecutionOutputBlock>,

    /// Return code of the script.
    ///
    /// 脚本执行返回码。
    pub return_code: i32,

    /// Standard error output.
    ///
    /// 标准错误输出。
    pub stderr: String,

    /// Standard stdout output.
    ///
    /// 标准 stdout 输出。
    pub stdout: String,
}

/// Encrypted code execution result block.
///
/// 加密代码执行结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedCodeExecutionResultBlock {
    /// Generated output file blocks.
    ///
    /// 生成的输出文件块。
    pub content: Vec<CodeExecutionOutputBlock>,

    /// Encrypted stdout string.
    ///
    /// 加密的 stdout 字符串。
    pub encrypted_stdout: String,

    /// Return code.
    ///
    /// 返回码。
    pub return_code: i32,

    /// Standard error output.
    ///
    /// 标准错误输出。
    pub stderr: String,
}

/// Code execution output file block.
///
/// 代码执行输出文件块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionOutputBlock {
    /// Output file ID.
    ///
    /// 输出文件 ID。
    pub file_id: StaticRefStr,
}

/// Bash code execution tool result block.
///
/// Bash 代码执行工具结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Execution output payload.
    ///
    /// 执行输出负载。
    pub content: BashCodeExecutionToolResultContent,
}

/// Content payload for Bash code execution tool result.
///
/// Bash 代码执行工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BashCodeExecutionToolResultContent {
    /// Error block.
    ///
    /// 错误块。
    Error(BashCodeExecutionToolResultError),

    /// Successful Bash execution result block.
    ///
    /// 成功的 Bash 执行结果块。
    Result(BashCodeExecutionResultBlock),
}

/// Error block for Bash code execution.
///
/// Bash 代码执行工具错误块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionToolResultError {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,
}

/// Bash code execution result block.
///
/// Bash 代码执行结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionResultBlock {
    /// Output file blocks.
    ///
    /// 输出文件块列表。
    pub content: Vec<BashCodeExecutionOutputBlock>,

    /// Process return code.
    ///
    /// 进程返回码。
    pub return_code: i32,

    /// Standard error string.
    ///
    /// 标准错误字符串。
    pub stderr: String,

    /// Standard stdout string.
    ///
    /// 标准 stdout 字符串。
    pub stdout: String,
}

/// Bash execution output file block.
///
/// Bash 执行输出文件块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionOutputBlock {
    /// File ID.
    ///
    /// 文件 ID。
    pub file_id: StaticRefStr,
}

/// Text editor code execution tool result block.
///
/// 文本编辑器代码执行工具结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Output result content payload.
    ///
    /// 输出结果内容负载。
    pub content: TextEditorCodeExecutionToolResultContent,
}

/// Content payload for text editor tool result.
///
/// 文本编辑器工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextEditorCodeExecutionToolResultContent {
    /// Execution error.
    ///
    /// 执行错误。
    Error(TextEditorCodeExecutionToolResultError),

    /// File view operation result.
    ///
    /// 文件查看操作结果。
    ViewResult(TextEditorCodeExecutionViewResultBlock),

    /// File creation operation result.
    ///
    /// 文件创建操作结果。
    CreateResult(TextEditorCodeExecutionCreateResultBlock),

    /// String replace operation result.
    ///
    /// 字符串替换操作结果。
    StrReplaceResult(TextEditorCodeExecutionStrReplaceResultBlock),
}

/// Error block for text editor tool.
///
/// 文本编辑器工具错误块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionToolResultError {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,

    /// Human readable error message.
    ///
    /// 可读的错误消息。
    pub error_message: String,
}

/// Result block for viewing a file in text editor tool.
///
/// 文本编辑器查看文件结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionViewResultBlock {
    /// File view text content.
    ///
    /// 查看的文件文本内容。
    pub content: String,

    /// File type ("text", "image", or "pdf").
    ///
    /// 文件类型。
    pub file_type: StaticRefStr,

    /// Number of lines displayed.
    ///
    /// 显示的行数。
    pub num_lines: u32,

    /// Start line offset.
    ///
    /// 起始行号。
    pub start_line: u32,

    /// Total lines in file.
    ///
    /// 文件总行数。
    pub total_lines: u32,
}

/// Result block for creating a file in text editor tool.
///
/// 文本编辑器创建文件结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionCreateResultBlock {
    /// Whether the file was updated rather than created anew.
    ///
    /// 是否为对已有文件的更新。
    pub is_file_update: bool,
}

/// Result block for string replace in text editor tool.
///
/// 文本编辑器字符串替换结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionStrReplaceResultBlock {
    /// Replaced lines content.
    ///
    /// 替换后的行内容列表。
    pub lines: Vec<String>,

    /// Number of new lines.
    ///
    /// 新行数。
    pub new_lines: u32,

    /// New start line offset.
    ///
    /// 新起始行号。
    pub old_lines: u32,

    /// Old start line offset.
    ///
    /// 原起始行号。
    pub old_start: u32,
}

/// Tool search tool result block.
///
/// 工具搜索工具结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultBlock {
    /// Tool use ID being responded to.
    ///
    /// 对应工具调用的 ID。
    pub tool_use_id: StaticRefStr,

    /// Result content payload.
    ///
    /// 结果内容负载。
    pub content: ToolSearchToolResultContent,
}

/// Content payload for tool search tool result.
///
/// 工具搜索工具结果的内容负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolSearchToolResultContent {
    /// Error during tool search execution.
    ///
    /// 工具搜索执行过程中的错误。
    Error(ToolSearchToolResultError),

    /// Successful tool search result block.
    ///
    /// 成功的工具搜索结果块。
    SearchResult(ToolSearchToolSearchResultBlock),
}

/// Error block for tool search tool.
///
/// 工具搜索工具错误块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultError {
    /// Error code string.
    ///
    /// 错误码。
    pub error_code: StaticRefStr,

    /// Human readable error message.
    ///
    /// 可读的错误消息。
    pub error_message: String,
}

/// Search result block containing matching tool references.
///
/// 包含匹配工具引用的搜索结果块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolSearchResultBlock {
    /// Matched tool reference blocks.
    ///
    /// 匹配到的工具引用块列表。
    pub tool_references: Vec<ToolReferenceBlock>,
}

/// Reference block pointing to a tool by name.
///
/// 按名称指向工具的引用块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReferenceBlock {
    /// Name of the referenced tool.
    ///
    /// 被引用的工具名称。
    pub tool_name: StaticRefStr,
}

/// Container upload block structure.
///
/// 容器上传文件内容块结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUploadBlock {
    /// ID of the uploaded file inside container.
    ///
    /// 容器内已上传文件的 ID。
    pub file_id: StaticRefStr,
}
