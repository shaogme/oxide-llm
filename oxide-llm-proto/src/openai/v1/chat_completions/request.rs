use super::{FunctionDefinition, Tool, ToolCall, ToolChoice};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// A list of messages comprising the conversation so far.
    pub messages: Vec<ChatCompletionMessage>,

    /// ID of the model to use.
    pub model: StaticRefStr,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<StaticRefStr, f32>>,

    /// Whether to return log probabilities of the output tokens or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position, each with an associated log probability.
    /// `logprobs` must be set to `true` if this parameter is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// The maximum number of tokens that can be generated in the chat completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// An upper bound for the number of tokens that can be generated for a completion, including visible output tokens and reasoning tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// How many chat completion choices to generate for each input message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,

    /// Output types that you would like the model to generate for this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<StaticRefStr>>,

    /// Configuration for a Predicted Output, which can greatly improve response times when large parts of the model response are known ahead of time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<PredictionContent>,

    /// Parameters for audio output. Required when audio output is requested with `modalities: ["audio"]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioOptions>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// An object specifying the format that the model must output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// This feature is in Beta. If specified, our system will make a best effort to sample deterministically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Specifies the latency tier to use for processing the request. This argument is currently available to standard tier customers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Whether or not to store the output of this chat completion request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// If set, partial message deltas will be sent, like in ChatGPT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming response. Only set this when you set `stream: true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A list of tools the model may call. Currently, only functions are supported as a tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Controls which (if any) tool is called by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>, // or ParallelToolCalls struct if complex

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<StaticRefStr>,

    /// Deprecated in favor of `tool_choice`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,

    /// Deprecated in favor of `tools`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,

    /// Parameters for web search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<StaticRefStr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum ChatCompletionMessage {
    System {
        content: StaticRefStr,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },
    User {
        content: UserContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<StaticRefStr>,
    },
    Tool {
        content: String,
        tool_call_id: StaticRefStr,
    },
    Developer {
        content: StaticRefStr,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    InputAudio { input_audio: InputAudio },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: StaticRefStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    pub data: StaticRefStr,
    pub format: StaticRefStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOptions {
    pub voice: StaticRefStr,
    pub format: StaticRefStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionContent {
    pub r#type: StaticRefStr,
    pub content: StaticRefStr, // Simplified, check spec if needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchemaDefinition },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    pub name: StaticRefStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,
    pub schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    String(StaticRefStr),
    Array(Vec<StaticRefStr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<u32>, // Approximation as u32 or generic Int/Enum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchUserLocation {
    pub r#type: StaticRefStr,
    pub approximate: StaticRefStr, // Or struct
}
