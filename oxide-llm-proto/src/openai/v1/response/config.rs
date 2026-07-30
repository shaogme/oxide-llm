use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for conversation state in OpenAI Response API.
///
/// OpenAI Response API 中的对话状态参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    /// Conversation ID string.
    ///
    /// 对话 ID 字符串。
    Id(StaticRefStr),
    /// Conversation parameter object containing an ID.
    ///
    /// 包含 ID 的对话参数对象。
    Object {
        /// The unique ID of the conversation.
        ///
        /// 对话的唯一标识符。
        id: StaticRefStr,
    },
}

/// Configuration options for reasoning models.
///
/// 推理模型的配置选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConf {
    /// Constrains effort on reasoning for reasoning models.
    ///
    /// 限制推理模型的推理努力程度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,

    /// A summary of the reasoning performed by the model.
    ///
    /// 模型执行的推理摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,

    /// Deprecated: use `summary` instead.
    ///
    /// 已废弃：请改用 `summary`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_summary: Option<ReasoningSummary>,
}

/// Constrains effort on reasoning for reasoning models.
///
/// 限制推理模型的推理努力程度。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    /// Disable reasoning effort.
    ///
    /// 禁用推理努力。
    None,
    /// Minimal reasoning effort.
    ///
    /// 极小程度的推理努力。
    Minimal,
    /// Low reasoning effort.
    ///
    /// 较低程度的推理努力。
    Low,
    /// Medium reasoning effort (default).
    ///
    /// 中等程度的推理努力（默认）。
    Medium,
    /// High reasoning effort.
    ///
    /// 较高程度的推理努力。
    High,
    /// Extra high reasoning effort.
    ///
    /// 超高程度的推理努力。
    Xhigh,
}

/// A summary of the reasoning performed by the model.
///
/// 模型执行的推理摘要生成模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
    /// Automatically determine reasoning summary level.
    ///
    /// 自动决定推理摘要级别。
    Auto,
    /// Concise reasoning summary.
    ///
    /// 简短的推理摘要。
    Concise,
    /// Detailed reasoning summary.
    ///
    /// 详细的推理摘要。
    Detailed,
}

/// Configuration options for a text response from the model.
///
/// 模型文本响应的配置选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextParam {
    /// Format specification for the model output text.
    ///
    /// 模型输出文本的格式规范。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextResponseFormatConfiguration>,

    /// Constrains the verbosity of the model's response.
    ///
    /// 限制模型响应的详细程度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
}

/// Constrains the verbosity of the model's response.
///
/// 限制模型响应的详细程度。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    /// Low verbosity for concise responses.
    ///
    /// 较低详细程度，获得更加简洁的响应。
    Low,
    /// Medium verbosity (default).
    ///
    /// 中等详细程度（默认）。
    Medium,
    /// High verbosity for detailed responses.
    ///
    /// 较高详细程度，获得更加详尽的响应。
    High,
}

/// Format configuration for text responses.
///
/// 文本响应的格式配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextResponseFormatConfiguration {
    /// Plain text format.
    ///
    /// 纯文本格式。
    Text,
    /// JSON object format.
    ///
    /// JSON 对象格式。
    JsonObject,
    /// JSON Schema response format.
    ///
    /// JSON Schema 响应格式。
    JsonSchema {
        /// The JSON Schema definition.
        ///
        /// JSON Schema 定义。
        json_schema: serde_json::Value,
    },
}

/// Additional output data to include in the model response.
///
/// 包含在模型响应中的附加输出数据选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeEnum {
    /// Include the search results of the file search tool call.
    ///
    /// 包含文件搜索工具调用的搜索结果。
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    /// Include the results of the web search tool call.
    ///
    /// 包含网络搜索工具调用的搜索结果。
    #[serde(rename = "web_search_call.results")]
    WebSearchCallResults,
    /// Include the sources of the web search tool call.
    ///
    /// 包含网络搜索工具调用的数据来源。
    #[serde(rename = "web_search_call.action.sources")]
    WebSearchCallActionSources,
    /// Include image URLs from the input message.
    ///
    /// 包含输入消息中的图片 URL。
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageImageUrl,
    /// Include image URLs from the computer call output.
    ///
    /// 包含计算机调用输出中的图片 URL。
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputOutputImageUrl,
    /// Include the outputs of Python code execution in code interpreter tool call items.
    ///
    /// 包含代码解释器工具调用中的 Python 代码执行输出。
    #[serde(rename = "code_interpreter_call.outputs")]
    CodeInterpreterCallOutputs,
    /// Include an encrypted version of reasoning tokens in reasoning item outputs.
    ///
    /// 包含推理项输出中的加密推理词元。
    #[serde(rename = "reasoning.encrypted_content")]
    ReasoningEncryptedContent,
    /// Include logprobs with assistant messages.
    ///
    /// 包含助手消息的对数概率信息。
    #[serde(rename = "message.output_text.logprobs")]
    MessageOutputTextLogprobs,
}

/// Reference to a prompt template and its variables.
///
/// 提示词模板及其变量引用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// The unique identifier of the prompt template to use.
    ///
    /// 要使用的提示词模板的唯一标识符。
    pub id: StaticRefStr,

    /// Optional version of the prompt template.
    ///
    /// 提示词模板的可选版本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<StaticRefStr>,

    /// Variables to substitute into the prompt template.
    ///
    /// 替换到提示词模板中的变量映射。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<StaticRefStr, serde_json::Value>>,
}

/// The truncation strategy to use for the model response.
///
/// 用于模型响应的截断策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truncation {
    /// Truncate the response automatically if input exceeds context window.
    ///
    /// 若输入超出上下文窗口，自动截断响应。
    Auto,
    /// Fail with an error if input exceeds context window (default).
    ///
    /// 若输入超出上下文窗口，直接返回错误（默认）。
    Disabled,
}
