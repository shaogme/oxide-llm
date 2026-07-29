use oxide_llm_proto::gemini::v1beta::generate_content::request::{
    GenerateContentRequest, GenerationConfig, SafetySetting,
};
use oxide_llm_proto::gemini::v1beta::generate_content::{Content, Tool as GeminiTool, ToolConfig};
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

    // Generation Config fields
    stop_sequences: Option<Vec<StaticRefStr>>,
    response_mime_type: Option<StaticRefStr>,
    max_output_tokens: Option<i32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    response_logprobs: Option<bool>,
    logprobs: Option<i32>,
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
            response_schema: None, // Can be added to optional config if needed
            candidate_count: None,
            max_output_tokens: self.optional.max_output_tokens,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            top_k: self.optional.top_k,
            seed: None,
            presence_penalty: self.optional.presence_penalty,
            frequency_penalty: self.optional.frequency_penalty,
            response_logprobs: self.optional.response_logprobs,
            logprobs: self.optional.logprobs,
            speech_config: None,
            thinking_config: None,
            image_config: None,
            media_resolution: None,
            response_json_schema: None,
            response_modalities: None,
        });

        GenerateContentRequest {
            contents,
            tools,
            tool_config: tool_config_override.or(self.optional.tool_config),
            safety_settings: self.optional.safety_settings,
            system_instruction: system_instruction_override.or(self.optional.system_instruction),
            generation_config,
            cached_content: self.optional.cached_content,
        }
    }
}
