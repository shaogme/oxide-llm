use super::{ConversationParam, Prompt, ReasoningConf, ResponseTextParam, Truncation};
use crate::openai::v1::{Tool, ToolChoice};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    /// Text, image, or file inputs to the model, used to generate a response.
    pub input: InputParam,

    /// ID of the model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StaticRefStr>,

    /// Specific output data to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<StaticRefStr>>,

    /// Whether to enable parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Whether to store the generated model response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// A system (or developer) message inserted into the model's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<StaticRefStr>,

    /// If set to true, the model response data will be streamed to the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ResponseStreamOptions>,

    /// Controls the conversation state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,

    /// Metadata to attach to the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<StaticRefStr, serde_json::Value>>,

    /// Number of most likely tokens to return at each token position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// Sampling temperature to use, between 0 and 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling probability mass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A list of tools the model may call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Controls which (if any) tool is called by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// User identifier (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<StaticRefStr>,

    /// Safety identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<StaticRefStr>,

    /// Prompt cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<StaticRefStr>,

    /// Service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,

    /// Prompt cache retention policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<StaticRefStr>,

    /// An upper bound for the number of tokens that can be generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// The maximum number of total calls to built-in tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,

    /// The unique ID of the previous response to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<StaticRefStr>,

    /// Configuration options for reasoning models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConf>,

    /// Whether to run the model response in the background.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    /// Configuration options for a text response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,

    /// Reference to a prompt template and its variables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,

    /// The truncation strategy to use for the model response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputParam {
    String(Cow<'static, str>),
    List(Vec<InputItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputItem {
    Message(InputMessage),
    // Add other input item types as needed based on InputItem schema
    // ReferenceParam(ItemReferenceParam),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub role: Cow<'static, str>,
    pub content: InputMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputMessageContent {
    String(String),
    Parts(Vec<InputContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContentPart {
    InputText { text: String },
    InputImage { image_url: StaticRefStr },
    InputAudio { input_audio: InputAudioContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioContent {
    pub data: StaticRefStr,
    pub format: StaticRefStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}
