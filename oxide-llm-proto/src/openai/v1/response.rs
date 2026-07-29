pub mod request;
#[allow(clippy::module_inception)]
pub mod response;

use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    Id(StaticRefStr),
    None, // Representing "off" or "auto" as generic json later or precise enums if known
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextResponseFormatConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<StaticRefStr>, // Placeholder for Verbosity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextResponseFormatConfiguration {
    Text,
    JsonSchema { json_schema: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: StaticRefStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<StaticRefStr, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truncation {
    Auto,
    Disabled,
}
