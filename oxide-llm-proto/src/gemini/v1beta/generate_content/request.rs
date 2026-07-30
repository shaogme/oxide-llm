use super::{
    Content, HarmBlockMethod, HarmBlockThreshold, HarmCategory, Modality, Schema, ServiceTier,
    Tool, ToolConfig,
};
use ref_str::StaticRefStr;
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
    pub cached_content: Option<StaticRefStr>,
    /// Optional. The service tier of the request.
    ///
    /// 可选。请求的服务层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Optional. Configures the logging behavior for a given request.
    ///
    /// 可选。配置给定请求的日志记录行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
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
    /// Optional. The method for blocking content.
    ///
    /// 可选。阻止内容的方法。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<HarmBlockMethod>,
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
    pub stop_sequences: Option<Vec<StaticRefStr>>,
    /// Optional. MIME type of the generated candidate text.
    ///
    /// 可选。生成的候选项文本的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<StaticRefStr>,
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
    pub response_modalities: Option<Vec<Modality>>,
    /// Optional. Enables enhanced civic answers.
    ///
    /// 可选。启用增强的公民回答。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,
    /// Optional. If enabled, the model will detect emotions and adapt its responses accordingly.
    ///
    /// 可选。如果启用，模型将检测情绪并相应地调整其响应。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_affective_dialog: Option<bool>,
    /// Optional. Configuration for the response output format.
    ///
    /// 可选。响应输出格式的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormatConfig>,
    /// Optional. Config for translation.
    ///
    /// 可选。翻译配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_config: Option<TranslationConfig>,
    /// Optional. Specifies the thinking level for the model.
    ///
    /// 可选。指定模型的思考层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
    /// Optional. Whether to include thought summaries in the response.
    ///
    /// 可选。是否在响应中包含思考摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_summaries: Option<ThinkingSummaries>,
    /// Optional. Config for video generation.
    ///
    /// 可选。视频生成的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_config: Option<VideoConfig>,
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
    pub voice_name: Option<StaticRefStr>,
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
    pub speaker: StaticRefStr,
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
    pub aspect_ratio: Option<StaticRefStr>,
    /// Optional. Specifies the size of generated images.
    ///
    /// 可选。指定生成的图像的大小。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<StaticRefStr>,
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

/// Configuration for the response output format.
///
/// 响应输出格式的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormatConfig {
    /// Optional. Text output format configuration.
    ///
    /// 可选。文本输出格式配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextResponseFormat>,
    /// Optional. Audio output format configuration.
    ///
    /// 可选。音频输出格式配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioResponseFormat>,
    /// Optional. Image output format configuration.
    ///
    /// 可选。图像输出格式配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageResponseFormat>,
}

/// Configuration for text output format.
///
/// 文本输出格式配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextResponseFormat {
    /// Optional. The MIME type of the text output.
    ///
    /// 可选。文本输出的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<TextResponseMimeType>,
    /// Optional. The JSON schema that the output should conform to.
    ///
    /// 可选。输出应符合的 JSON 模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Supported MIME types for text output.
///
/// 文本输出支持的 MIME 类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextResponseMimeType {
    MimeTypeUnspecified,
    ApplicationJson,
    TextPlain,
}

/// Configuration for audio output format.
///
/// 音频输出格式配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioResponseFormat {
    /// Optional. The MIME type of the audio output.
    ///
    /// 可选。音频输出的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<AudioResponseMimeType>,
    /// Optional. The delivery mode for the audio output.
    ///
    /// 可选。音频输出的交付模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<Delivery>,
    /// Optional. Sample rate in Hz.
    ///
    /// 可选。采样率（Hz）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    /// Optional. Bit rate in bits per second (bps).
    ///
    /// 可选。比特率（bps）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
}

/// Supported MIME types for audio output.
///
/// 音频输出支持的 MIME 类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioResponseMimeType {
    MimeTypeUnspecified,
    AudioMp3,
    AudioOggOpus,
    AudioL16,
    AudioWav,
    AudioAlaw,
    AudioMulaw,
}

/// Delivery mode for output.
///
/// 输出的交付模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Delivery {
    DeliveryUnspecified,
    Inline,
    Uri,
}

/// Configuration for image output format.
///
/// 图像输出格式配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponseFormat {
    /// Optional. The MIME type of the image output.
    ///
    /// 可选。图像输出的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<ImageResponseMimeType>,
    /// Optional. The delivery mode for the image output.
    ///
    /// 可选。图像输出的交付模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<Delivery>,
    /// Optional. The aspect ratio for the image output.
    ///
    /// 可选。图像输出的长宽比。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatio>,
    /// Optional. The size of the image output.
    ///
    /// 可选。图像输出的大小。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<ImageSize>,
}

/// Supported MIME types for image output.
///
/// 图像输出支持的 MIME 类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageResponseMimeType {
    MimeTypeUnspecified,
    ImageJpeg,
}

/// Supported aspect ratios for image output.
///
/// 图像输出支持的长宽比。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AspectRatio {
    AspectRatioUnspecified,
    AspectRatioOneByOne,
    AspectRatioTwoByThree,
    AspectRatioThreeByTwo,
    AspectRatioThreeByFour,
    AspectRatioFourByThree,
    AspectRatioFourByFive,
    AspectRatioFiveByFour,
    AspectRatioNineBySixteen,
    AspectRatioSixteenByNine,
    AspectRatioTwentyOneByNine,
    AspectRatioOneByEight,
    AspectRatioEightByOne,
    AspectRatioOneByFour,
    AspectRatioFourByOne,
}

/// Supported image sizes for image output.
///
/// 图像输出支持的图像大小。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageSize {
    ImageSizeUnspecified,
    ImageSizeFiveTwelve,
    ImageSizeOneK,
    ImageSizeTwoK,
    ImageSizeFourK,
}

/// Config for translation features.
///
/// 翻译功能配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationConfig {
    /// Required. The target language for translation.
    ///
    /// 必填。翻译的目标语言。
    pub target_language_code: StaticRefStr,
    /// Optional. If true, the model will generate audio when target language is spoken.
    ///
    /// 可选。如果为 true，模型将在说目标语言时生成音频。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo_target_language: Option<bool>,
}

/// Whether to include thought summaries in the response.
///
/// 是否在响应中包含思考摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingSummaries {
    ThinkingSummariesUnspecified,
    ThinkingSummariesAuto,
    ThinkingSummariesNone,
}

/// Config for video generation features.
///
/// 视频生成功能的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoConfig {
    /// Optional. Task mode for video generation.
    ///
    /// 可选。视频生成的任务模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<Task>,
}

/// Supported video generation tasks.
///
/// 支持的视频生成任务。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Task {
    TaskUnspecified,
    TextToVideo,
    ImageToVideo,
    ReferenceToVideo,
    Edit,
}


