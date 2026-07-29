pub mod chat_completions;
pub mod response;

use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: StaticRefStr,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: StaticRefStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    String(StaticRefStr), // "none", "auto", "required"
    Named(ToolChoiceNamed),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceNamed {
    pub r#type: StaticRefStr,
    pub function: ToolChoiceFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    pub name: StaticRefStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: StaticRefStr,
    pub r#type: StaticRefStr,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: StaticRefStr,
    pub arguments: StaticRefStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    pub content: Option<Vec<LogProbToken>>,
    pub refusal: Option<Vec<LogProbToken>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbToken {
    pub token: StaticRefStr,
    pub logprob: f32,
    pub bytes: Option<Vec<u8>>,
    #[serde(default)]
    pub top_logprobs: Vec<LogProbToken>,
}
