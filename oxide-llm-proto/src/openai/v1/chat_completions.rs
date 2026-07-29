pub mod chunk;
pub mod request;
pub mod response;

pub use crate::openai::v1::{FunctionDefinition, ToolChoiceFunction};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: StaticRefStr,
    pub function: FunctionDefinition,
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
