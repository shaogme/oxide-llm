use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::container::Container;
use super::content::{Content, ContentBlock, Role};
use super::thinking::ThinkingConfigParam;
use super::tool::{Tool, ToolChoice};

/// Request object for the Messages API (`POST /v1/messages`).
///
/// Messages API 的请求对象 (`POST /v1/messages`)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    /// The model that will complete your prompt.
    ///
    /// 将补全提示词的目标模型名称。
    pub model: StaticRefStr,

    /// Input messages.
    ///
    /// 输入消息列表。
    pub messages: Vec<Message>,

    /// The maximum number of tokens to generate before stopping.
    ///
    /// 停止之前生成的最大 token 数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// System prompt.
    ///
    /// 系统提示词。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Object describing metadata about the request.
    ///
    /// 描述请求元数据的对象。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// Information about the container used in the request.
    ///
    /// 请求中使用的容器信息（代码执行工具使用）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<Container>,

    /// Custom text sequences that will cause the model to stop generating.
    ///
    /// 导致模型停止生成的自定义文本序列。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<StaticRefStr>>,

    /// Whether to incrementally stream the response using server-sent events.
    ///
    /// 是否使用 SSE 流式增量返回响应。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Amount of randomness injected into the response.
    ///
    /// 注入响应中的随机程度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// How the model should use the provided tools.
    ///
    /// 模型应如何使用所提供的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Definitions of tools that the model may use.
    ///
    /// 模型可能使用的工具定义列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Only sample from the top K options for each subsequent token.
    ///
    /// 每个后续 token 仅从概率前 K 个选项中采样。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Use nucleus sampling.
    ///
    /// 使用核采样。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Configuration for enabling extended thinking.
    ///
    /// 启用深度思考模式的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfigParam>,

    /// Configuration options for the model's output.
    ///
    /// 模型输出的格式与思考强度配置选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,

    /// The service tier to use (e.g. "auto", "standard_only").
    ///
    /// 使用的服务层级（如 "auto", "standard_only"）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,
}

/// An individual message turn in a request.
///
/// 请求中的单个消息轮次。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Conversational role ("user" or "assistant").
    ///
    /// 对话角色（"user" 或 "assistant"）。
    pub role: Role,

    /// Message content payload.
    ///
    /// 消息内容负载。
    pub content: Content,
}

/// System prompt, specified as a string or a vector of content blocks.
///
/// 系统提示词，可为字符串或内容块数组。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Text system prompt string.
    ///
    /// 文本形式的系统提示词。
    Text(StaticRefStr),

    /// Block system prompt vector.
    ///
    /// 内容块数组形式的系统提示词。
    Blocks(Vec<ContentBlock>),
}

/// Metadata attached to a message request.
///
/// 附加在消息请求上的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// User ID tracking identifier.
    ///
    /// 用户 ID 追踪标识。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<StaticRefStr>,
}

/// Output configuration for model reasoning effort and structured JSON output.
///
/// 模型输出配置（包含思考强度与结构化 JSON 格式）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Reasoning effort level ("low", "medium", "high", "xhigh", "max").
    ///
    /// 推理思考强度等级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<OutputEffort>,

    /// Output format parameter.
    ///
    /// 输出格式参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
}

/// Effort level for model reasoning output.
///
/// 模型推理输出的思考强度等级。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputEffort {
    /// Low reasoning effort.
    ///
    /// 低强度思考。
    Low,

    /// Medium reasoning effort.
    ///
    /// 中等强度思考。
    Medium,

    /// High reasoning effort.
    ///
    /// 高强度思考。
    High,

    /// Extra high reasoning effort.
    ///
    /// 超高强度思考。
    Xhigh,

    /// Maximum reasoning effort.
    ///
    /// 最高强度思考。
    Max,
}

/// Output format payload.
///
/// 输出格式负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    /// Format type ("json_schema").
    ///
    /// 格式类型（固定为 "json_schema"）。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// JSON Schema defining expected output structure.
    ///
    /// 定义预期输出结构的 JSON Schema。
    pub schema: Value,
}
