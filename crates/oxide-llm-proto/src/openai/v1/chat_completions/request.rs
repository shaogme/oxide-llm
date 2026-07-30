use super::{FunctionDefinition, Tool, ToolCall, ToolChoice};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request payload for creating a chat completion.
///
/// 创建聊天补全的请求载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// A list of messages comprising the conversation so far.
    ///
    /// 包含迄今为止对话的消息列表。
    pub messages: Vec<ChatCompletionMessage>,

    /// ID of the model to use.
    ///
    /// 要使用的模型 ID。
    pub model: StaticRefStr,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far.
    ///
    /// 介于 -2.0 和 2.0 之间的数字。正值根据文本中现有词元的出现频率惩罚新词元。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    ///
    /// 修改指定 token 在补全中出现的可能性。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<StaticRefStr, f32>>,

    /// Whether to return log probabilities of the output tokens or not.
    ///
    /// 是否返回输出 token 的对数概率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position.
    ///
    /// 介于 0 和 20 之间的整数，指定在每个 token 位置返回的最可能 token 数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// The maximum number of tokens that can be generated in the chat completion (deprecated in favor of `max_completion_tokens`).
    ///
    /// 在聊天补全中可生成的最大 token 数（已废弃，推荐使用 `max_completion_tokens`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// An upper bound for the number of tokens that can be generated for a completion, including visible output tokens and reasoning tokens.
    ///
    /// 为补全生成的 token 数量上限，包含可见输出 token 与推理 token 。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// How many chat completion choices to generate for each input message.
    ///
    /// 为每个输入消息生成多少个聊天补全选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,

    /// Output types that you would like the model to generate for this request.
    ///
    /// 希望模型为此请求生成的输出类型模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<StaticRefStr>>,

    /// Configuration for a Predicted Output.
    ///
    /// 预测输出（Predicted Output）的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<PredictionContent>,

    /// Parameters for audio output.
    ///
    /// 音频输出的参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioOptions>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far.
    ///
    /// 介于 -2.0 和 2.0 之间的数字。正值根据新 token 至今是否出现过进行惩罚。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// An object specifying the format that the model must output.
    ///
    /// 指定模型必须输出的格式的对象。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Sampling deterministically seed.
    ///
    /// 确定性采样的随机种子。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Specifies the latency tier to use for processing the request.
    ///
    /// 指定用于处理请求的服务延迟层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    ///
    /// API 将停止生成后续 token 的最多 4 个停止序列。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Whether or not to store the output of this chat completion request.
    ///
    /// 是否存储此聊天补全请求的输出。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// If set, partial message deltas will be sent as server-sent events.
    ///
    /// 若设置，将以 SSE 服务端发送事件形式发送增量消息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming response.
    ///
    /// 流式响应的选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// What sampling temperature to use, between 0 and 2.
    ///
    /// 使用的采样温度，介于 0 和 2 之间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling threshold.
    ///
    /// 核采样的概率阈值（Top-p）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A list of tools the model may call.
    ///
    /// 模型可以调用的工具列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Controls which (if any) tool is called by the model.
    ///
    /// 控制模型调用哪个（如果有）工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel tool calls.
    ///
    /// 是否启用并行工具调用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// A unique identifier representing your end-user.
    ///
    /// 代表最终用户的唯一标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<StaticRefStr>,

    /// Deprecated in favor of `tool_choice`.
    ///
    /// 已废弃，请改用 `tool_choice`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,

    /// Deprecated in favor of `tools`.
    ///
    /// 已废弃，请改用 `tools`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,

    /// Parameters for web search tool.
    ///
    /// 网络搜索工具参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,

    /// Verbosity level for output.
    ///
    /// 输出详细程度控制。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<StaticRefStr>,

    /// Constrains effort on reasoning for reasoning models.
    ///
    /// 限制推理模型的推理努力程度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Metadata attached to request.
    ///
    /// 附加到请求的元数据键值对。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<StaticRefStr, StaticRefStr>>,
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

/// A message comprising the conversation so far in a chat completion request.
///
/// 聊天补全请求中包含迄今为止对话的消息枚举。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum ChatCompletionMessage {
    /// Developer-provided instructions (replaces system message for o1 and newer models).
    ///
    /// 开发者指令（在 o1 及更新模型中替代系统消息）。
    Developer {
        /// Content of the developer message.
        ///
        /// 开发者消息内容。
        content: DeveloperContent,

        /// Optional name for the participant.
        ///
        /// 参与者的可选名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },

    /// System instructions provided by the developer.
    ///
    /// 开发者提供的系统指令。
    System {
        /// Content of the system message.
        ///
        /// 系统消息内容。
        content: SystemContent,

        /// Optional name for the participant.
        ///
        /// 参与者的可选名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },

    /// Messages sent by an end user.
    ///
    /// 最终用户发送的消息。
    User {
        /// Content of the user message.
        ///
        /// 用户消息内容。
        content: UserContent,

        /// Optional name for the participant.
        ///
        /// 参与者的可选名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,
    },

    /// Messages sent by the model in response to user messages.
    ///
    /// 模型回应用户消息所发送的消息。
    Assistant {
        /// Text content of the assistant message.
        ///
        /// 助手消息的文本内容。
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<StaticRefStr>,

        /// Optional name for the participant.
        ///
        /// 参与者的可选名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<StaticRefStr>,

        /// Tool calls generated by the model.
        ///
        /// 模型生成的工具调用。
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,

        /// Refusal message generated by the model.
        ///
        /// 模型生成的拒绝回答文本。
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<StaticRefStr>,

        /// Data about a previous audio response from the model.
        ///
        /// 模型此前音频响应的数据。
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<AssistantAudio>,

        /// Deprecated function call generated by the model.
        ///
        /// 模型生成的函数调用（已废弃）。
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<serde_json::Value>,
    },

    /// Message responding to a tool call.
    ///
    /// 响应工具调用的消息。
    Tool {
        /// Content of the tool message.
        ///
        /// 工具消息内容。
        content: ToolContent,

        /// Tool call ID that this message is responding to.
        ///
        /// 此消息响应的工具调用 ID。
        tool_call_id: StaticRefStr,
    },

    /// Deprecated function message.
    ///
    /// 函数消息（已废弃）。
    Function {
        /// Content of the function message.
        ///
        /// 函数消息内容。
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<StaticRefStr>,

        /// The name of the function called.
        ///
        /// 被调用的函数名称。
        name: StaticRefStr,
    },
}

/// Content of a developer message.
///
/// 开发者消息的内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeveloperContent {
    /// Plain text developer instructions.
    ///
    /// 纯文本开发者指令。
    Text(StaticRefStr),

    /// Array of content parts.
    ///
    /// 内容块组件数组。
    Parts(Vec<ContentPart>),
}

impl From<StaticRefStr> for DeveloperContent {
    fn from(s: StaticRefStr) -> Self {
        DeveloperContent::Text(s)
    }
}

/// Content of a system message.
///
/// 系统消息的内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContent {
    /// Plain text system instructions.
    ///
    /// 纯文本系统指令。
    Text(StaticRefStr),

    /// Array of content parts.
    ///
    /// 内容块组件数组。
    Parts(Vec<ContentPart>),
}

impl From<StaticRefStr> for SystemContent {
    fn from(s: StaticRefStr) -> Self {
        SystemContent::Text(s)
    }
}

/// Content of a user message.
///
/// 用户消息的内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    /// Plain text user input.
    ///
    /// 纯文本用户输入。
    Text(String),

    /// Array of content parts (text, image, audio, file).
    ///
    /// 多模态内容块组件数组（文本、图片、音频、文件）。
    Parts(Vec<ContentPart>),
}

impl From<String> for UserContent {
    fn from(s: String) -> Self {
        UserContent::Text(s)
    }
}

/// Content of a tool response message.
///
/// 工具响应消息的内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolContent {
    /// Plain text tool response output.
    ///
    /// 纯文本工具响应输出。
    Text(String),

    /// Array of content parts for tool output.
    ///
    /// 工具输出的内容块组件数组。
    Parts(Vec<ContentPart>),
}

impl From<String> for ToolContent {
    fn from(s: String) -> Self {
        ToolContent::Text(s)
    }
}

/// A multimodal content part in a message.
///
/// 消息中的多模态内容块组件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content part.
    ///
    /// 文本内容块。
    Text { text: String },

    /// Image URL or Base64 content part.
    ///
    /// 图片 URL 或 Base64 内容块。
    ImageUrl { image_url: ImageUrl },

    /// Input audio content part.
    ///
    /// 音频输入内容块。
    InputAudio { input_audio: InputAudio },

    /// File input content part.
    ///
    /// 文件输入内容块。
    File { file: FileData },

    /// Refusal content part.
    ///
    /// 拒绝回答内容块。
    Refusal { refusal: String },
}

/// Image content part parameters.
///
/// 图片内容块参数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Either a URL of the image or base64 encoded image data.
    ///
    /// 图片的 URL 地址或 Base64 编码数据。
    pub url: StaticRefStr,

    /// Detail level of the image ("auto", "low", "high").
    ///
    /// 图片的细节清晰程度级别（"auto"、"low"、"high"）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<StaticRefStr>,
}

/// Input audio content part parameters.
///
/// 音频输入内容块参数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64 encoded audio data.
    ///
    /// Base64 编码的音频数据。
    pub data: StaticRefStr,

    /// Format of the audio data ("wav" or "mp3").
    ///
    /// 音频数据的格式（"wav" 或 "mp3"）。
    pub format: StaticRefStr,
}

/// File content part parameters.
///
/// 文件内容块参数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileData {
    /// The name of the file.
    ///
    /// 文件名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<StaticRefStr>,

    /// The base64 encoded file data.
    ///
    /// Base64 编码的文件数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<StaticRefStr>,

    /// The ID of an uploaded file to use as input.
    ///
    /// 已上传文件的 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<StaticRefStr>,
}

/// Assistant audio response reference in request message.
///
/// 请求消息中对助手此前音频响应的引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistantAudio {
    /// Unique identifier for a previous audio response from the model.
    ///
    /// 模型此前音频响应的唯一标识符。
    pub id: StaticRefStr,
}

/// Audio output configuration parameters.
///
/// 音频输出配置参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOptions {
    /// The voice the model uses to respond (e.g. "alloy", "echo", etc.).
    ///
    /// 模型用于回答的音色（如 "alloy"、"echo" 等）。
    pub voice: StaticRefStr,

    /// Specifies the output audio format (e.g. "mp3", "opus").
    ///
    /// 指定输出音频的格式（如 "mp3"、"opus" 等）。
    pub format: StaticRefStr,
}

/// Configuration for Predicted Output.
///
/// 预测输出（Predicted Output）配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionContent {
    /// The type of predicted content.
    ///
    /// 预测内容的类型。
    pub r#type: StaticRefStr,

    /// The content of the prediction.
    ///
    /// 预测的内容文本。
    pub content: StaticRefStr,
}

/// An object specifying the response format for the model output.
///
/// 指定模型输出响应格式的对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Standard text output.
    ///
    /// 标准文本输出。
    Text,

    /// JSON Object mode output.
    ///
    /// JSON 对象模式输出。
    JsonObject,

    /// Structured Outputs with a JSON schema definition.
    ///
    /// 包含 JSON Schema 定义的结构化输出（Structured Outputs）。
    JsonSchema { json_schema: JsonSchemaDefinition },
}

/// JSON Schema definition for structured output.
///
/// 结构化输出的 JSON Schema 定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    /// The name of the response schema.
    ///
    /// 响应 Schema 的名称。
    pub name: StaticRefStr,

    /// A description of what the response schema is for.
    ///
    /// 响应 Schema 的功能说明描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,

    /// The schema for the response, specified as a JSON Schema object.
    ///
    /// 以 JSON Schema 对象形式指定的响应架构。
    pub schema: serde_json::Value,

    /// Whether to enable strict schema adherence.
    ///
    /// 是否启用严格模式遵循 Schema。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Stop sequence configuration for completion.
///
/// 补全的停止序列配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    /// A single stop string.
    ///
    /// 单个停止字符串。
    String(StaticRefStr),

    /// An array of stop strings (up to 4).
    ///
    /// 停止字符串数组（最多 4 个）。
    Array(Vec<StaticRefStr>),
}

/// Options for streaming response.
///
/// 流式响应的选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// If set, an additional chunk will be streamed with usage statistics.
    ///
    /// 若设置，将额外发送一个包含用量统计的流式 Chunk。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,

    /// When true, stream obfuscation will be enabled.
    ///
    /// 为 true 时将启用流混淆以抵御侧信道攻击。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}

/// Parameters for web search.
///
/// 网络搜索参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOptions {
    /// Approximate location parameters for search.
    ///
    /// 搜索的大致位置参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,

    /// Size of search context.
    ///
    /// 搜索上下文的大小。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<u32>,
}

/// Approximate location parameters for web search.
///
/// 网络搜索的大致位置参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchUserLocation {
    /// The type of location approximation (always "approximate").
    ///
    /// 位置近似类型（固定为 "approximate"）。
    pub r#type: StaticRefStr,

    /// Approximate location string or value.
    ///
    /// 近似位置字符串或数值。
    pub approximate: StaticRefStr,
}
