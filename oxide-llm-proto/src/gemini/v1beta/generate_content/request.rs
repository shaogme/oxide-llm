use super::{Content, HarmBlockThreshold, HarmCategory, Schema, Tool, ToolConfig};
use serde::{Deserialize, Serialize};

/// Generates a model response given an input `GenerateContentRequest`.
///
/// 生成给定输入 `GenerateContentRequest` 的模型响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentRequest {
    /// Required. The content of the current conversation with the model.
    ///
    /// 必填。与模型当前对话的内容。
    pub contents: Vec<Content>,
    /// Optional. A list of `Tools` the `Model` may use to generate the next response.
    ///
    /// 可选。`Model` 可以用来生成下一个响应的 `Tools` 列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Optional. Tool configuration for any `Tool` specified in the request.
    ///
    /// 可选。请求中指定的任何 `Tool` 的工具配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    /// Optional. A list of unique `SafetySetting` instances for blocking unsafe content.
    ///
    /// 可选。用于阻止不安全内容的唯一 `SafetySetting` 实例列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    /// Optional. Developer set system instruction(s). Currently, text only.
    ///
    /// 可选。开发者设置的系统指令。目前仅限文本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    /// Optional. Configuration options for model generation and outputs.
    ///
    /// 可选。模型生成和输出的配置选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    /// Optional. The name of the content cached to use as context to serve the prediction.
    ///
    /// 可选。缓存内容的名称，用作提供预测的上下文。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
}

/// Safety setting, affecting the safety-blocking behavior.
///
/// 安全设置，影响安全阻止行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    /// Required. The category for this setting.
    ///
    /// 必填。此设置的类别。
    pub category: HarmCategory,
    /// Required. Controls the probability threshold at which harm is blocked.
    ///
    /// 必填。控制阻止伤害的概率阈值。
    pub threshold: HarmBlockThreshold,
}

/// Configuration options for model generation and outputs.
///
/// 模型生成和输出的配置选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    /// Optional. The set of character sequences (up to 5) that will stop output generation.
    ///
    /// 可选。将停止输出生成的字符序列集（最多 5 个）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Optional. MIME type of the generated candidate text.
    ///
    /// 可选。生成的候选项文本的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    /// Optional. Output schema of the generated candidate text.
    ///
    /// 可选。生成的候选项文本的输出模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Schema>,
    /// Optional. Number of generated responses to return.
    ///
    /// 可选。要返回的生成的响应数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    /// Optional. The maximum number of tokens to include in a response candidate.
    ///
    /// 可选。响应候选项中包含的最大令牌数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    /// Optional. Controls the randomness of the output.
    ///
    /// 可选。控制输出的随机性。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional. The maximum cumulative probability of tokens to consider when sampling.
    ///
    /// 可选。采样时要考虑的令牌的最大累积概率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Optional. The maximum number of tokens to consider when sampling.
    ///
    /// 可选。采样时要考虑的最大令牌数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    /// Optional. Seed used in decoding.
    ///
    /// 可选。解码中使用的种子。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    /// Optional. Presence penalty applied to the next token's logprobs.
    ///
    /// 可选。应用于下一个令牌的 logprobs 的存在惩罚。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Optional. Frequency penalty applied to the next token's logprobs.
    ///
    /// 可选。应用于下一个令牌的 logprobs 的频率惩罚。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Optional. If true, export the logprobs results in response.
    ///
    /// 可选。如果为 true，则在响应中导出 logprobs 结果。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
    /// Optional. This sets the number of top logprobs to return at each decoding step.
    ///
    /// 可选。这将设置每个解码步骤要返回的顶部 logprobs 的数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,
    /// Optional. The speech generation config.
    ///
    /// 可选。语音生成配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    /// Optional. Config for thinking features.
    ///
    /// 可选。思考功能的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
    /// Optional. Config for image generation.
    ///
    /// 可选。图像生成的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<ImageConfig>,
    /// Optional. If specified, the media resolution specified will be used.
    ///
    /// 可选。如果指定，将使用指定的媒体分辨率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<MediaResolution>,
    /// Optional. Output schema of the generated response (JSON Schema).
    ///
    /// 可选。生成的响应的输出模式（JSON Schema）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<serde_json::Value>,
    /// Optional. The requested modalities of the response.
    ///
    /// 可选。响应的请求模态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<super::Modality>>,
}

/// The speech generation config.
///
/// 语音生成配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechConfig {
    /// The configuration in case of single-voice output.
    ///
    /// 单语音输出的情况下的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,
    /// Optional. The configuration for the multi-speaker setup.
    ///
    /// 可选。多扬声器设置的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_speaker_voice_config: Option<MultiSpeakerVoiceConfig>,
}

/// The configuration for the voice to use.
///
/// 要使用的语音的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    /// The configuration for the prebuilt voice to use.
    ///
    /// 要使用的预构建语音的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilt_voice_config: Option<PrebuiltVoiceConfig>,
}

/// The configuration for the prebuilt speaker to use.
///
/// 要使用的预构建扬声器的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrebuiltVoiceConfig {
    /// The name of the preset voice to use.
    ///
    /// 要使用的预设语音的名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_name: Option<String>,
}

/// The configuration for the multi-speaker setup.
///
/// 多扬声器设置的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSpeakerVoiceConfig {
    /// Required. All the enabled speaker voices.
    ///
    /// 必填。所有启用的扬声器语音。
    pub speaker_voice_configs: Vec<SpeakerVoiceConfig>,
}

/// The configuration for a single speaker in a multi speaker setup.
///
/// 多扬声器设置中单个扬声器的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerVoiceConfig {
    /// Required. The name of the speaker to use.
    ///
    /// 必填。要使用的扬声器的名称。
    pub speaker: String,
    /// Required. The configuration for the voice to use.
    ///
    /// 必填。要使用的语音的配置。
    pub voice_config: VoiceConfig,
}

/// Config for thinking features.
///
/// 思考功能的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    /// Indicates whether to include thoughts in the response.
    ///
    /// 指示是否在响应中包含思考。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,
    /// The number of thoughts tokens that the model should generate.
    ///
    /// 模型应生成的思考令牌数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
    /// Optional. Controls the maximum depth of the model's internal reasoning process.
    ///
    /// 可选。控制模型内部推理过程的最大深度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

/// Allow user to specify how much to think using enum instead of integer budget.
///
/// 允许用户使用枚举而不是整数预算来指定思考的程度。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingLevel {
    ThinkingLevelUnspecified,
    Minimal,
    Low,
    Medium,
    High,
}

/// Config for image generation features.
///
/// 图像生成功能的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    /// Optional. The aspect ratio of the image to generate.
    ///
    /// 可选。要生成的图像的长宽比。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Optional. Specifies the size of generated images.
    ///
    /// 可选。指定生成的图像的大小。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<String>,
}

/// Media resolution for the input media.
///
/// 输入媒体的媒体分辨率。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaResolution {
    MediaResolutionUnspecified,
    MediaResolutionLow,
    MediaResolutionMedium,
    MediaResolutionHigh,
}
