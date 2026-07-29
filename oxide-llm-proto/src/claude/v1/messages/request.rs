use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CacheControl, Content, ContentBlock, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    /// The model that will complete your prompt.
    pub model: StaticRefStr,

    /// Input messages.
    pub messages: Vec<Message>,

    /// The maximum number of tokens to generate before stopping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// System prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Object describing metadata about the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// Custom text sequences that will cause the model to stop generating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<StaticRefStr>>,

    /// Whether to incrementally stream the response using server-sent events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Amount of randomness injected into the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// How the model should use the provided tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Definitions of tools that the model may use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Only sample from the top K options for each subsequent token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Use nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Configuration for enabling thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    /// Configuration options for the model's output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,

    /// The service tier to use (e.g. "auto", "standard_only").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(StaticRefStr),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ToolChoice {
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Tool {
        name: StaticRefStr,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tool {
    Custom(CustomTool),
    Bash(BashTool),
    TextEditor(TextEditorTool),
    WebSearch(WebSearchTool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    pub name: StaticRefStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub r#type: Option<StaticRefStr>, // "custom"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool {
    pub name: StaticRefStr, // "bash"
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "bash_20250124"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool {
    pub name: StaticRefStr, // "str_replace_editor" or "str_replace_based_edit_tool"
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "text_editor_20250124", "text_editor_20250429", "text_editor_20250728"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// Maximum number of characters to display when viewing a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    pub name: StaticRefStr, // "web_search"
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "web_search_20250305"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<StaticRefStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<StaticRefStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchUserLocation {
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "approximate"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "json_schema"
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub r#type: StaticRefStr, // "enabled"
    pub budget_tokens: u32,
}
