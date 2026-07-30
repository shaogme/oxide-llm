use super::{
    ConversationParam, Prompt, ReasoningConf, ResponseTextParam, Tool, ToolChoice, Truncation,
};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

/// Request payload for creating a model response in OpenAI Response API.
///
/// OpenAI Response API 中创建模型响应的请求载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    /// Text, image, or file inputs to the model, used to generate a response.
    ///
    /// 输入给模型的文本、图片或文件，用于生成响应。
    pub input: InputParam,

    /// ID of the model to use.
    ///
    /// 要使用的模型 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StaticRefStr>,

    /// Specific output data to include in the model response.
    ///
    /// 模型响应中要包含的特定附加数据类型列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<StaticRefStr>>,

    /// Whether to enable parallel tool calls.
    ///
    /// 是否允许模型并行发起工具调用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Whether to store the generated model response for later retrieval.
    ///
    /// 是否存储生成的模型响应以供后续获取。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// A system (or developer) message inserted into the model's context.
    ///
    /// 插入模型上下文中的系统（或开发者）消息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<StaticRefStr>,

    /// If set to true, the model response data will be streamed to the client.
    ///
    /// 若设置为 true，模型响应数据将通过 SSE 流式返回客户端。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming responses.
    ///
    /// 流式响应的额外选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ResponseStreamOptions>,

    /// Controls the conversation state.
    ///
    /// 控制对话状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,

    /// Metadata to attach to the response.
    ///
    /// 附加到响应上的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<StaticRefStr, serde_json::Value>>,

    /// Number of most likely tokens to return at each token position.
    ///
    /// 在每个词元位置返回的最可能词元数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// Sampling temperature to use, between 0 and 2.
    ///
    /// 采样温度，介于 0 和 2 之间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling probability mass.
    ///
    /// 核采样概率质量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A list of tools the model may call.
    ///
    /// 模型可调用的工具列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Controls which (if any) tool is called by the model.
    ///
    /// 控制模型要调用的工具（如果有）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// User identifier (deprecated).
    ///
    /// 用户标识符（已废弃）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<StaticRefStr>,

    /// Safety identifier for content moderation and safety checks.
    ///
    /// 用于内容审核和安全检查的安全标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<StaticRefStr>,

    /// Prompt cache key to boost cache hit rates.
    ///
    /// 用于提升缓存命中率的提示词缓存键。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<StaticRefStr>,

    /// Service tier for processing the request.
    ///
    /// 处理请求的服务层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<StaticRefStr>,

    /// Prompt cache retention policy.
    ///
    /// 提示词缓存保留策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<StaticRefStr>,

    /// An upper bound for the number of tokens that can be generated.
    ///
    /// 可生成的词元数量上限。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// The maximum number of total calls to built-in tools.
    ///
    /// 内置工具调用的最大总次数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,

    /// The unique ID of the previous response to the model.
    ///
    /// 模型前一次响应的唯一 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<StaticRefStr>,

    /// Configuration options for reasoning models.
    ///
    /// 推理模型的配置选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConf>,

    /// Whether to run the model response in the background.
    ///
    /// 是否在后台运行模型响应生成。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    /// Configuration options for a text response.
    ///
    /// 文本响应的配置选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,

    /// Reference to a prompt template and its variables.
    ///
    /// 提示词模板及其变量引用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,

    /// The truncation strategy to use for the model response.
    ///
    /// 用于模型响应的截断策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
}

/// Text, image, or file inputs to the model.
///
/// 输入给模型的文本、图片或文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputParam {
    /// Direct string prompt input.
    ///
    /// 直接传入的文本提示词字符串。
    String(Cow<'static, str>),

    /// List of input items.
    ///
    /// 输入项构成的列表。
    List(Vec<InputItem>),
}

/// An individual item representing input context to the model.
///
/// 代表模型输入上下文的单个项目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    /// An input message item.
    ///
    /// 输入消息项。
    Message(InputMessage),

    /// A function call item made previously by assistant.
    ///
    /// 助手此前发起的函数调用项。
    FunctionCall {
        /// Unique call ID.
        ///
        /// 调用的唯一 ID。
        call_id: StaticRefStr,

        /// Function name.
        ///
        /// 函数名称。
        name: StaticRefStr,

        /// JSON arguments string.
        ///
        /// JSON 字符串格式的函数参数。
        arguments: String,

        /// Item ID.
        ///
        /// 项目 ID。
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<StaticRefStr>,
    },

    /// The output of a function call.
    ///
    /// 函数调用的输出结果项。
    FunctionCallOutput {
        /// Call ID matching the function call.
        ///
        /// 对应的函数调用 ID。
        call_id: StaticRefStr,

        /// Output string returned by tool execution.
        ///
        /// 工具执行返回的输出字符串。
        output: String,

        /// Item ID.
        ///
        /// 项目 ID。
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<StaticRefStr>,
    },
}

/// A message input to the model with a role.
///
/// 带有角色的模型输入消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    /// Role of the message (e.g. user, system, developer).
    ///
    /// 消息的角色（如 user, system, developer）。
    pub role: Cow<'static, str>,

    /// Contents of the message.
    ///
    /// 消息的内容。
    pub content: InputMessageContent,

    /// Optional author name.
    ///
    /// 可选的作者名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,

    /// Status of the item.
    ///
    /// 项目的状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Cow<'static, str>>,
}

/// Content representation of an input message.
///
/// 输入消息的内容表达形式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputMessageContent {
    /// Plain string content.
    ///
    /// 纯字符串内容。
    String(String),

    /// Multi-part structured content parts.
    ///
    /// 由多部分构成的结构化内容列表。
    Parts(Vec<InputContentPart>),
}

/// A content part within a multi-modal input message.
///
/// 多模态输入消息中的内容部分。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContentPart {
    /// Text content part.
    ///
    /// 文本内容部分。
    InputText {
        /// Text string.
        ///
        /// 文本内容。
        text: String,
    },

    /// Image content part.
    ///
    /// 图片内容部分。
    InputImage {
        /// Image URL string or base64 data URI.
        ///
        /// 图片 URL 字符串或 Base64 数据 URI。
        image_url: StaticRefStr,
    },

    /// Audio content part.
    ///
    /// 音频内容部分。
    InputAudio {
        /// Input audio content object.
        ///
        /// 输入音频内容对象。
        input_audio: InputAudioContent,
    },
}

/// Input audio data content.
///
/// 输入音频数据内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioContent {
    /// Base64-encoded audio bytes.
    ///
    /// Base64 编码的音频数据。
    pub data: StaticRefStr,

    /// Format of the audio data (e.g., wav, mp3).
    ///
    /// 音频数据格式（如 wav, mp3）。
    pub format: StaticRefStr,
}

/// Streaming options for Response API request.
///
/// Response API 请求的流式处理选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStreamOptions {
    /// Whether to include obfuscated sequence tokens.
    ///
    /// 是否在流中包含混淆词元序列。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}
