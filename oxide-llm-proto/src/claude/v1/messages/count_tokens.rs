use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::request::{Message, SystemPrompt};
use super::thinking::ThinkingConfigParam;
use super::tool::{Tool, ToolChoice};

/// Request object for the Count Tokens API.
///
/// Count Tokens API 的请求对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountTokensRequest {
    /// Input messages.
    ///
    /// 输入消息列表。
    pub messages: Vec<Message>,

    /// The model to count tokens for.
    ///
    /// 计算 token 数量的目标模型。
    pub model: StaticRefStr,

    /// System prompt.
    ///
    /// 系统提示词。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Definitions of tools that the model may use.
    ///
    /// 模型可能使用的工具定义。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// How the model should use the provided tools.
    ///
    /// 模型应如何使用所提供的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Configuration for enabling thinking.
    ///
    /// 启用思考模式的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfigParam>,
}

/// Response object for the Count Tokens API.
///
/// Count Tokens API 的响应对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTokensCount {
    /// The total number of input tokens.
    ///
    /// 输入 token 的总数。
    pub input_tokens: u32,
}
