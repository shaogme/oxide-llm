use crate::{
    config::{Config, OptionalConfig, ReasoningEffort, RequiredConfig},
    error::AgentError,
};
use oxide_llm_proto::gemini::v1beta::generate_content::{
    Content, Modality, Schema, ServiceTier, Tool as GeminiTool, ToolConfig,
    request::{
        GenerateContentRequest, GenerationConfig, ImageConfig, MediaResolution,
        ResponseFormatConfig, SafetySetting, SpeechConfig, ThinkingConfig, ThinkingLevel,
        ThinkingSummaries, TranslationConfig, VideoConfig,
    },
};
use ref_str::StaticRefStr;

/// Configuration for Gemini Agent (Required).
///
/// Gemini 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct GenerateContentRequiredConfig {
    model: StaticRefStr,
    endpoint: StaticRefStr,
}

impl GenerateContentRequiredConfig {
    /// Create a new `GenerateContentRequiredConfig`.
    pub fn new(model: impl Into<StaticRefStr>, endpoint: impl Into<StaticRefStr>) -> Self {
        Self {
            model: model.into(),
            endpoint: endpoint.into(),
        }
    }

    /// Get model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Set model name.
    pub fn set_model(&mut self, model: impl Into<StaticRefStr>) -> &mut Self {
        self.model = model.into();
        self
    }

    /// Set model name (builder pattern).
    pub fn with_model(mut self, model: impl Into<StaticRefStr>) -> Self {
        self.model = model.into();
        self
    }

    /// Get endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Set endpoint.
    pub fn set_endpoint(&mut self, endpoint: impl Into<StaticRefStr>) -> &mut Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set endpoint (builder pattern).
    pub fn with_endpoint(mut self, endpoint: impl Into<StaticRefStr>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

/// Configuration for Gemini Agent (Optional).
///
/// Gemini 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct GenerateContentOptionalConfig {
    safety_settings: Option<Vec<SafetySetting>>,
    system_instruction: Option<Content>,
    tool_config: Option<ToolConfig>,
    cached_content: Option<StaticRefStr>,
    service_tier: Option<ServiceTier>,
    store: Option<bool>,

    // Generation Config fields
    stop_sequences: Option<Vec<StaticRefStr>>,
    response_mime_type: Option<StaticRefStr>,
    response_schema: Option<Schema>,
    candidate_count: Option<i32>,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    seed: Option<i32>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    response_logprobs: Option<bool>,
    logprobs: Option<i32>,
    speech_config: Option<SpeechConfig>,
    thinking_config: Option<ThinkingConfig>,
    image_config: Option<ImageConfig>,
    media_resolution: Option<MediaResolution>,
    response_json_schema: Option<serde_json::Value>,
    response_modalities: Option<Vec<Modality>>,
    enable_enhanced_civic_answers: Option<bool>,
    enable_affective_dialog: Option<bool>,
    response_format: Option<ResponseFormatConfig>,
    translation_config: Option<TranslationConfig>,
    thinking_level: Option<ThinkingLevel>,
    thinking_summaries: Option<ThinkingSummaries>,
    video_config: Option<VideoConfig>,
}

impl GenerateContentOptionalConfig {
    /// Create a new `GenerateContentOptionalConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get safety settings.
    pub fn safety_settings(&self) -> Option<&[SafetySetting]> {
        self.safety_settings.as_deref()
    }

    /// Set safety settings.
    pub fn set_safety_settings(
        &mut self,
        safety_settings: Option<Vec<SafetySetting>>,
    ) -> &mut Self {
        self.safety_settings = safety_settings;
        self
    }

    /// Set safety settings (builder pattern).
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Get system instruction.
    pub fn system_instruction(&self) -> Option<&Content> {
        self.system_instruction.as_ref()
    }

    /// Set system instruction.
    pub fn set_system_instruction(&mut self, system_instruction: Option<Content>) -> &mut Self {
        self.system_instruction = system_instruction;
        self
    }

    /// Set system instruction (builder pattern).
    pub fn with_system_instruction(mut self, system_instruction: Content) -> Self {
        self.system_instruction = Some(system_instruction);
        self
    }

    /// Get tool config.
    pub fn tool_config(&self) -> Option<&ToolConfig> {
        self.tool_config.as_ref()
    }

    /// Set tool config.
    pub fn set_tool_config(&mut self, tool_config: Option<ToolConfig>) -> &mut Self {
        self.tool_config = tool_config;
        self
    }

    /// Set tool config (builder pattern).
    pub fn with_tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.tool_config = Some(tool_config);
        self
    }

    /// Get cached content.
    pub fn cached_content(&self) -> Option<&str> {
        self.cached_content.as_deref()
    }

    /// Set cached content.
    pub fn set_cached_content(
        &mut self,
        cached_content: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.cached_content = cached_content.map(Into::into);
        self
    }

    /// Set cached content (builder pattern).
    pub fn with_cached_content(mut self, cached_content: impl Into<StaticRefStr>) -> Self {
        self.cached_content = Some(cached_content.into());
        self
    }

    /// Get service tier.
    pub fn service_tier(&self) -> Option<&ServiceTier> {
        self.service_tier.as_ref()
    }

    /// Set service tier.
    pub fn set_service_tier(&mut self, service_tier: Option<ServiceTier>) -> &mut Self {
        self.service_tier = service_tier;
        self
    }

    /// Set service tier (builder pattern).
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Get store.
    pub fn store(&self) -> Option<bool> {
        self.store
    }

    /// Set store.
    pub fn set_store(&mut self, store: Option<bool>) -> &mut Self {
        self.store = store;
        self
    }

    /// Set store (builder pattern).
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Get stop sequences.
    pub fn stop_sequences(&self) -> Option<&[StaticRefStr]> {
        self.stop_sequences.as_deref()
    }

    /// Set stop sequences.
    pub fn set_stop_sequences(&mut self, stop_sequences: Option<Vec<StaticRefStr>>) -> &mut Self {
        self.stop_sequences = stop_sequences;
        self
    }

    /// Set stop sequences (builder pattern).
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<StaticRefStr>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Get response mime type.
    pub fn response_mime_type(&self) -> Option<&str> {
        self.response_mime_type.as_deref()
    }

    /// Set response mime type.
    pub fn set_response_mime_type(
        &mut self,
        response_mime_type: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.response_mime_type = response_mime_type.map(Into::into);
        self
    }

    /// Set response mime type (builder pattern).
    pub fn with_response_mime_type(mut self, response_mime_type: impl Into<StaticRefStr>) -> Self {
        self.response_mime_type = Some(response_mime_type.into());
        self
    }

    /// Get response schema.
    pub fn response_schema(&self) -> Option<&Schema> {
        self.response_schema.as_ref()
    }

    /// Set response schema.
    pub fn set_response_schema(&mut self, response_schema: Option<Schema>) -> &mut Self {
        self.response_schema = response_schema;
        self
    }

    /// Set response schema (builder pattern).
    pub fn with_response_schema(mut self, response_schema: Schema) -> Self {
        self.response_schema = Some(response_schema);
        self
    }

    /// Get candidate count.
    pub fn candidate_count(&self) -> Option<i32> {
        self.candidate_count
    }

    /// Set candidate count.
    pub fn set_candidate_count(&mut self, candidate_count: Option<i32>) -> &mut Self {
        self.candidate_count = candidate_count;
        self
    }

    /// Set candidate count (builder pattern).
    pub fn with_candidate_count(mut self, candidate_count: i32) -> Self {
        self.candidate_count = Some(candidate_count);
        self
    }

    /// Get max output tokens.
    pub fn max_output_tokens(&self) -> Option<i32> {
        self.max_output_tokens
    }

    /// Set max output tokens.
    pub fn set_max_output_tokens(&mut self, max_output_tokens: Option<i32>) -> &mut Self {
        self.max_output_tokens = max_output_tokens;
        self
    }

    /// Set max output tokens (builder pattern).
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Get temperature.
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Set temperature.
    pub fn set_temperature(&mut self, temperature: Option<f32>) -> &mut Self {
        self.temperature = temperature;
        self
    }

    /// Set temperature (builder pattern).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Get top_p.
    pub fn top_p(&self) -> Option<f32> {
        self.top_p
    }

    /// Set top_p.
    pub fn set_top_p(&mut self, top_p: Option<f32>) -> &mut Self {
        self.top_p = top_p;
        self
    }

    /// Set top_p (builder pattern).
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Get top_k.
    pub fn top_k(&self) -> Option<i32> {
        self.top_k
    }

    /// Set top_k.
    pub fn set_top_k(&mut self, top_k: Option<i32>) -> &mut Self {
        self.top_k = top_k;
        self
    }

    /// Set top_k (builder pattern).
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Get seed.
    pub fn seed(&self) -> Option<i32> {
        self.seed
    }

    /// Set seed.
    pub fn set_seed(&mut self, seed: Option<i32>) -> &mut Self {
        self.seed = seed;
        self
    }

    /// Set seed (builder pattern).
    pub fn with_seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Get presence penalty.
    pub fn presence_penalty(&self) -> Option<f32> {
        self.presence_penalty
    }

    /// Set presence penalty.
    pub fn set_presence_penalty(&mut self, presence_penalty: Option<f32>) -> &mut Self {
        self.presence_penalty = presence_penalty;
        self
    }

    /// Set presence penalty (builder pattern).
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Get frequency penalty.
    pub fn frequency_penalty(&self) -> Option<f32> {
        self.frequency_penalty
    }

    /// Set frequency penalty.
    pub fn set_frequency_penalty(&mut self, frequency_penalty: Option<f32>) -> &mut Self {
        self.frequency_penalty = frequency_penalty;
        self
    }

    /// Set frequency penalty (builder pattern).
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Get response logprobs.
    pub fn response_logprobs(&self) -> Option<bool> {
        self.response_logprobs
    }

    /// Set response logprobs.
    pub fn set_response_logprobs(&mut self, response_logprobs: Option<bool>) -> &mut Self {
        self.response_logprobs = response_logprobs;
        self
    }

    /// Set response logprobs (builder pattern).
    pub fn with_response_logprobs(mut self, response_logprobs: bool) -> Self {
        self.response_logprobs = Some(response_logprobs);
        self
    }

    /// Get logprobs.
    pub fn logprobs(&self) -> Option<i32> {
        self.logprobs
    }

    /// Set logprobs.
    pub fn set_logprobs(&mut self, logprobs: Option<i32>) -> &mut Self {
        self.logprobs = logprobs;
        self
    }

    /// Set logprobs (builder pattern).
    pub fn with_logprobs(mut self, logprobs: i32) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Get speech config.
    pub fn speech_config(&self) -> Option<&SpeechConfig> {
        self.speech_config.as_ref()
    }

    /// Set speech config.
    pub fn set_speech_config(&mut self, speech_config: Option<SpeechConfig>) -> &mut Self {
        self.speech_config = speech_config;
        self
    }

    /// Set speech config (builder pattern).
    pub fn with_speech_config(mut self, speech_config: SpeechConfig) -> Self {
        self.speech_config = Some(speech_config);
        self
    }

    /// Get thinking config.
    pub fn thinking_config(&self) -> Option<&ThinkingConfig> {
        self.thinking_config.as_ref()
    }

    /// Set thinking config.
    pub fn set_thinking_config(&mut self, thinking_config: Option<ThinkingConfig>) -> &mut Self {
        self.thinking_config = thinking_config;
        self
    }

    /// Set thinking config (builder pattern).
    pub fn with_thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        self.thinking_config = Some(thinking_config);
        self
    }

    /// Get image config.
    pub fn image_config(&self) -> Option<&ImageConfig> {
        self.image_config.as_ref()
    }

    /// Set image config.
    pub fn set_image_config(&mut self, image_config: Option<ImageConfig>) -> &mut Self {
        self.image_config = image_config;
        self
    }

    /// Set image config (builder pattern).
    pub fn with_image_config(mut self, image_config: ImageConfig) -> Self {
        self.image_config = Some(image_config);
        self
    }

    /// Get media resolution.
    pub fn media_resolution(&self) -> Option<&MediaResolution> {
        self.media_resolution.as_ref()
    }

    /// Set media resolution.
    pub fn set_media_resolution(&mut self, media_resolution: Option<MediaResolution>) -> &mut Self {
        self.media_resolution = media_resolution;
        self
    }

    /// Set media resolution (builder pattern).
    pub fn with_media_resolution(mut self, media_resolution: MediaResolution) -> Self {
        self.media_resolution = Some(media_resolution);
        self
    }

    /// Get response json schema.
    pub fn response_json_schema(&self) -> Option<&serde_json::Value> {
        self.response_json_schema.as_ref()
    }

    /// Set response json schema.
    pub fn set_response_json_schema(
        &mut self,
        response_json_schema: Option<serde_json::Value>,
    ) -> &mut Self {
        self.response_json_schema = response_json_schema;
        self
    }

    /// Set response json schema (builder pattern).
    pub fn with_response_json_schema(mut self, response_json_schema: serde_json::Value) -> Self {
        self.response_json_schema = Some(response_json_schema);
        self
    }

    /// Get response modalities.
    pub fn response_modalities(&self) -> Option<&[Modality]> {
        self.response_modalities.as_deref()
    }

    /// Set response modalities.
    pub fn set_response_modalities(
        &mut self,
        response_modalities: Option<Vec<Modality>>,
    ) -> &mut Self {
        self.response_modalities = response_modalities;
        self
    }

    /// Set response modalities (builder pattern).
    pub fn with_response_modalities(mut self, response_modalities: Vec<Modality>) -> Self {
        self.response_modalities = Some(response_modalities);
        self
    }

    /// Get enable enhanced civic answers.
    pub fn enable_enhanced_civic_answers(&self) -> Option<bool> {
        self.enable_enhanced_civic_answers
    }

    /// Set enable enhanced civic answers.
    pub fn set_enable_enhanced_civic_answers(
        &mut self,
        enable_enhanced_civic_answers: Option<bool>,
    ) -> &mut Self {
        self.enable_enhanced_civic_answers = enable_enhanced_civic_answers;
        self
    }

    /// Set enable enhanced civic answers (builder pattern).
    pub fn with_enable_enhanced_civic_answers(
        mut self,
        enable_enhanced_civic_answers: bool,
    ) -> Self {
        self.enable_enhanced_civic_answers = Some(enable_enhanced_civic_answers);
        self
    }

    /// Get enable affective dialog.
    pub fn enable_affective_dialog(&self) -> Option<bool> {
        self.enable_affective_dialog
    }

    /// Set enable affective dialog.
    pub fn set_enable_affective_dialog(
        &mut self,
        enable_affective_dialog: Option<bool>,
    ) -> &mut Self {
        self.enable_affective_dialog = enable_affective_dialog;
        self
    }

    /// Set enable affective dialog (builder pattern).
    pub fn with_enable_affective_dialog(mut self, enable_affective_dialog: bool) -> Self {
        self.enable_affective_dialog = Some(enable_affective_dialog);
        self
    }

    /// Get response format.
    pub fn response_format(&self) -> Option<&ResponseFormatConfig> {
        self.response_format.as_ref()
    }

    /// Set response format.
    pub fn set_response_format(
        &mut self,
        response_format: Option<ResponseFormatConfig>,
    ) -> &mut Self {
        self.response_format = response_format;
        self
    }

    /// Set response format (builder pattern).
    pub fn with_response_format(mut self, response_format: ResponseFormatConfig) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Get translation config.
    pub fn translation_config(&self) -> Option<&TranslationConfig> {
        self.translation_config.as_ref()
    }

    /// Set translation config.
    pub fn set_translation_config(
        &mut self,
        translation_config: Option<TranslationConfig>,
    ) -> &mut Self {
        self.translation_config = translation_config;
        self
    }

    /// Set translation config (builder pattern).
    pub fn with_translation_config(mut self, translation_config: TranslationConfig) -> Self {
        self.translation_config = Some(translation_config);
        self
    }

    /// Get thinking level.
    pub fn thinking_level(&self) -> Option<&ThinkingLevel> {
        self.thinking_level.as_ref()
    }

    /// Set thinking level.
    pub fn set_thinking_level(&mut self, thinking_level: Option<ThinkingLevel>) -> &mut Self {
        self.thinking_level = thinking_level;
        self
    }

    /// Set thinking level (builder pattern).
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        self.thinking_level = Some(thinking_level);
        self
    }

    /// Get thinking summaries.
    pub fn thinking_summaries(&self) -> Option<&ThinkingSummaries> {
        self.thinking_summaries.as_ref()
    }

    /// Set thinking summaries.
    pub fn set_thinking_summaries(
        &mut self,
        thinking_summaries: Option<ThinkingSummaries>,
    ) -> &mut Self {
        self.thinking_summaries = thinking_summaries;
        self
    }

    /// Set thinking summaries (builder pattern).
    pub fn with_thinking_summaries(mut self, thinking_summaries: ThinkingSummaries) -> Self {
        self.thinking_summaries = Some(thinking_summaries);
        self
    }

    /// Get video config.
    pub fn video_config(&self) -> Option<&VideoConfig> {
        self.video_config.as_ref()
    }

    /// Set video config.
    pub fn set_video_config(&mut self, video_config: Option<VideoConfig>) -> &mut Self {
        self.video_config = video_config;
        self
    }

    /// Set video config (builder pattern).
    pub fn with_video_config(mut self, video_config: VideoConfig) -> Self {
        self.video_config = Some(video_config);
        self
    }
}

/// Configuration for Gemini Agent.
///
/// Gemini 代理配置。
#[derive(Debug, Clone)]
pub struct GenerateContentConfig {
    required: GenerateContentRequiredConfig,
    optional: GenerateContentOptionalConfig,
}

impl GenerateContentConfig {
    /// Create a new `GenerateContentConfig`.
    pub fn new(required: GenerateContentRequiredConfig) -> Self {
        Self {
            required,
            optional: GenerateContentOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    pub fn required(&self) -> &GenerateContentRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    pub fn required_mut(&mut self) -> &mut GenerateContentRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    pub fn optional(&self) -> &GenerateContentOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    pub fn optional_mut(&mut self) -> &mut GenerateContentOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    pub fn with_optional(mut self, optional: GenerateContentOptionalConfig) -> Self {
        self.optional = optional;
        self
    }

    /// Convert Config to GenerateContentRequest with provided contents.
    ///
    /// 将配置转换为 GenerateContentRequest，并填入内容。
    pub fn to_request(
        self,
        contents: Vec<Content>,
        system_instruction_override: Option<Content>,
        tools: Option<Vec<GeminiTool>>,
        tool_config_override: Option<ToolConfig>,
    ) -> GenerateContentRequest {
        let generation_config = Some(GenerationConfig {
            stop_sequences: self.optional.stop_sequences,
            response_mime_type: self.optional.response_mime_type,
            response_schema: self.optional.response_schema,
            candidate_count: self.optional.candidate_count,
            max_output_tokens: self.optional.max_output_tokens,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            top_k: self.optional.top_k,
            seed: self.optional.seed,
            presence_penalty: self.optional.presence_penalty,
            frequency_penalty: self.optional.frequency_penalty,
            response_logprobs: self.optional.response_logprobs,
            logprobs: self.optional.logprobs,
            speech_config: self.optional.speech_config,
            thinking_config: self.optional.thinking_config,
            image_config: self.optional.image_config,
            media_resolution: self.optional.media_resolution,
            response_json_schema: self.optional.response_json_schema,
            response_modalities: self.optional.response_modalities,
            enable_enhanced_civic_answers: self.optional.enable_enhanced_civic_answers,
            enable_affective_dialog: self.optional.enable_affective_dialog,
            response_format: self.optional.response_format,
            translation_config: self.optional.translation_config,
            thinking_level: self.optional.thinking_level,
            thinking_summaries: self.optional.thinking_summaries,
            video_config: self.optional.video_config,
        });

        GenerateContentRequest {
            contents,
            tools,
            tool_config: tool_config_override.or(self.optional.tool_config),
            safety_settings: self.optional.safety_settings,
            system_instruction: system_instruction_override.or(self.optional.system_instruction),
            generation_config,
            cached_content: self.optional.cached_content,
            service_tier: self.optional.service_tier,
            store: self.optional.store,
        }
    }
}

impl crate::agent::builder::AgentConfigTrait for GenerateContentConfig {
    type Required = GenerateContentRequiredConfig;
    type Optional = GenerateContentOptionalConfig;

    fn from_required(required: Self::Required) -> Self {
        Self::new(required)
    }

    fn with_optional(self, optional: Self::Optional) -> Self {
        self.with_optional(optional)
    }
}

impl TryFrom<RequiredConfig> for GenerateContentRequiredConfig {
    type Error = AgentError;

    fn try_from(config: RequiredConfig) -> Result<Self, Self::Error> {
        let model = config
            .model_static()
            .ok_or_else(|| AgentError::Config("model is required".into()))?;
        let endpoint = config
            .endpoint_static()
            .ok_or_else(|| AgentError::Config("endpoint is required".into()))?;

        Ok(Self::new(model, endpoint))
    }
}

impl TryFrom<OptionalConfig> for GenerateContentOptionalConfig {
    type Error = AgentError;

    fn try_from(config: OptionalConfig) -> Result<Self, Self::Error> {
        let OptionalConfig {
            temperature,
            top_p,
            top_k,
            frequency_penalty,
            presence_penalty,
            stop_sequences,
            seed,
            reasoning_effort,
        } = config;

        let mut optional = Self::new();
        if let Some(temp) = temperature {
            optional.set_temperature(Some(temp));
        }
        if let Some(top_p) = top_p {
            optional.set_top_p(Some(top_p));
        }
        if let Some(top_k) = top_k {
            optional.set_top_k(Some(top_k as i32));
        }
        if let Some(freq_pen) = frequency_penalty {
            optional.set_frequency_penalty(Some(freq_pen));
        }
        if let Some(pres_pen) = presence_penalty {
            optional.set_presence_penalty(Some(pres_pen));
        }
        if let Some(stop) = stop_sequences {
            optional.set_stop_sequences(Some(stop));
        }
        if let Some(seed) = seed {
            optional.set_seed(Some(seed as i32));
        }
        if let Some(effort) = reasoning_effort {
            let level = match effort {
                ReasoningEffort::None => ThinkingLevel::Minimal,
                ReasoningEffort::Minimal => ThinkingLevel::Minimal,
                ReasoningEffort::Low => ThinkingLevel::Low,
                ReasoningEffort::Medium => ThinkingLevel::Medium,
                ReasoningEffort::High => ThinkingLevel::High,
                ReasoningEffort::Xhigh => ThinkingLevel::High,
                ReasoningEffort::Max => ThinkingLevel::High,
            };
            optional.set_thinking_level(Some(level));
        }
        Ok(optional)
    }
}

impl TryFrom<Config> for GenerateContentConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let (required, optional) = (config.required().clone(), config.optional().clone());
        let required = GenerateContentRequiredConfig::try_from(required)?;
        let optional = GenerateContentOptionalConfig::try_from(optional)?;
        if let Some(max_tokens) = config.required().max_tokens() {
            let mut optional = optional;
            optional.set_max_output_tokens(Some(max_tokens as i32));
            Ok(Self::new(required).with_optional(optional))
        } else {
            Ok(Self::new(required).with_optional(optional))
        }
    }
}
