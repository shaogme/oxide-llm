use super::{
    ConversationParam, LogProbs, Prompt, ReasoningConf, ResponseTextParam, Tool, ToolChoice,
    Truncation,
};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 表示 OpenAI Response API 返回的模型响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// 此 Response 的唯一标识符。
    pub id: String,

    /// 此资源的对象类型 - 固定为 `response`。
    pub object: String,

    /// 响应生成的状态。
    pub status: ResponseStatus,

    /// 此 Response 创建时的 Unix 时间戳（秒）。
    pub created_at: i64,

    /// 此 Response 完成时的 Unix 时间戳（秒）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,

    /// 若响应失败时的错误详细信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,

    /// 响应未完成的原因详细信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<IncompleteDetails>,

    /// 模型生成的输出项数组。
    #[serde(default)]
    pub output: Vec<OutputItem>,

    /// 插入上下文的指令。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// 附加到响应上的键值对元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// 用于生成响应的模型 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 最高对数概率选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// 使用的采样温度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// 核采样 top_p 值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// 用户标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// 用于请求的服务层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    /// 代表后端配置的系统指纹。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,

    /// 响应的词元使用详细统计。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,

    /// 请求中提供的工具列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// 请求中指定的工具选择策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// 是否允许并行工具调用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// 与响应关联的对话参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,

    /// 前一次响应的唯一 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,

    /// 推理配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConf>,

    /// 响应是否在后台执行。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    /// 最大输出词元约束。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// 最大工具调用次数约束。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,

    /// 文本响应配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,

    /// 提示词模板引用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,

    /// 使用的截断策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
}

/// 响应生成的状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// 生成成功完成。
    Completed,
    /// 生成失败。
    Failed,
    /// 生成进行中。
    InProgress,
    /// 生成已被取消。
    Cancelled,
    /// 生成正在排队中。
    Queued,
    /// 生成未完成。
    Incomplete,
}

/// 响应错误详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// 错误代码字符串。
    pub code: String,

    /// 可读的错误消息内容。
    pub message: String,
}

/// 解释响应为何未完成的详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteDetails {
    /// 导致未完成状态的原因字符串。
    pub reason: String,
}

/// 作为模型响应一部分生成的输出项目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    /// 助手消息输出项。
    Message(OutputMessage),
    /// 文件搜索工具调用项。
    FileSearchCall(FileSearchToolCall),
    /// 函数工具调用项。
    FunctionCall(FunctionToolCall),
    /// 网络搜索工具调用项。
    WebSearchCall(WebSearchToolCall),
    /// 计算机工具调用项。
    ComputerCall(ComputerToolCall),
    /// 包含思考/推理内容的推理项。
    Reasoning(ReasoningItem),
    /// 压解/压缩主体项。
    CompactionBody(GenericToolCallItem),
    /// 图片生成工具调用项。
    ImageGenCall(ImageGenToolCall),
    /// 代码解释器工具调用项。
    CodeInterpreterCall(CodeInterpreterToolCall),
    /// 本地 Shell 工具调用项。
    LocalShellCall(GenericToolCallItem),
    /// 函数 Shell 工具调用项。
    FunctionShellCall(GenericToolCallItem),
    /// 函数 Shell 调用输出项。
    FunctionShellCallOutput(GenericToolCallItem),
    /// 应用补丁调用项。
    ApplyPatchCall(GenericToolCallItem),
    /// 应用补丁调用输出项。
    ApplyPatchCallOutput(GenericToolCallItem),
    /// MCP 工具调用项。
    McpToolCall(GenericToolCallItem),
    /// MCP 工具列表查询项。
    McpListTools(GenericToolCallItem),
    /// MCP 审批请求项。
    McpApprovalRequest(GenericToolCallItem),
    /// 自定义工具调用项。
    CustomToolCall(GenericToolCallItem),
}

/// 通用工具调用结构（用于自定义或扩展工具项）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericToolCallItem {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 对应工具请求的调用 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,

    /// 执行状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 工具名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 模型生成的输出消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMessage {
    /// 输出消息的唯一标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 消息的角色（固定为 "assistant"）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// 构成该消息的内容块列表。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<OutputMessageContent>,

    /// 消息状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,
}

/// 输出消息的内容块部分。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputMessageContent {
    /// 文本内容输出。
    OutputText(OutputTextContent),

    /// 拒答内容输出。
    Refusal(RefusalContent),
}

/// 带有可选标注与对数概率的文本内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTextContent {
    /// 生成的文本内容字符串。
    pub text: String,

    /// 标注信息列表（如引用信息）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<AnnotationItem>>,

    /// 输出词元的对数概率（若请求）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProbs>>,
}

/// 附加到输出文本上的标注信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationItem {
    /// 标注类型。
    pub r#type: String,

    /// 引用或标注的文本字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// 在输出文本中的起始字符索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<usize>,

    /// 在输出文本中的结束字符索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<usize>,
}

/// 拒答消息输出内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalContent {
    /// 拒答消息内容。
    pub refusal: StaticRefStr,
}

/// 特定词元类型的详细使用统计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    /// 从提示词缓存中获取的词元数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,

    /// 模型用于思考/推理的词元数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    /// 生成的文本词元数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u32>,

    /// 生成或消耗的音频词元数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// 响应词元用量详细统计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    /// 请求使用的词元总数（输入 + 输出）。
    pub total_tokens: u32,

    /// 处理的输入词元数量。
    pub input_tokens: u32,

    /// 生成的输出词元数量。
    pub output_tokens: u32,

    /// 输入词元的详细分解。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<TokenDetails>,

    /// 输出词元的详细分解。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_token_details: Option<TokenDetails>,
}

/// 网络搜索工具调用项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 搜索执行状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 网络搜索执行的查询语句列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<String>>,

    /// 返回的网络搜索结果列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<WebSearchResult>>,
}

/// 单个网络搜索结果条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    /// 网页标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// 网页的 URL。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// 网页摘录片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// 模型生成的函数工具调用项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 调用的唯一 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,

    /// 函数调用项的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 要调用的函数名称。
    #[serde(default)]
    pub name: String,

    /// JSON 格式的函数参数字符串。
    #[serde(default)]
    pub arguments: String,
}

/// 文件搜索工具调用项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 文件搜索调用的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 文件搜索使用的查询列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<String>>,
}

/// 计算机工具调用项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 计算机工具调用的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 计算机工具执行的动作名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// 包含模型推理词元和摘要的推理项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningItem {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 推理项的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 若启用时的加密推理内容字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,

    /// 推理摘要内容部分列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Vec<ReasoningContentPart>>,

    /// 推理详细内容部分列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ReasoningContentPart>>,
}

/// 推理项的内容部分。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningContentPart {
    /// 推理内容部分的类型（如 text）。
    pub r#type: String,

    /// 推理内容的文本字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// 图片生成工具调用项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 图片生成工具调用的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,
}

/// 代码解释器工具调用项输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCall {
    /// 项目 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 代码解释器调用的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// 代码执行环境的容器 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    /// 执行的 Python 代码字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// 代码执行产生的输出列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<ReasoningContentPart>>,
}
