use oxide_llm_proto::claude::v1::messages::request::{
    Message as ClaudeMessage, MessagesRequest, OutputConfig, SystemPrompt, ThinkingConfig, Tool,
    ToolChoice,
};
use ref_str::StaticRefStr;

/// Configuration for Claude Messages Agent (Required).
///
/// Claude Messages 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct MessagesRequiredConfig {
    model: StaticRefStr,
    max_tokens: u32,
    endpoint: StaticRefStr,
}

impl MessagesRequiredConfig {
    /// Create a new `MessagesRequiredConfig`.
    pub fn new(
        model: impl Into<StaticRefStr>,
        max_tokens: u32,
        endpoint: impl Into<StaticRefStr>,
    ) -> Self {
        Self {
            model: model.into(),
            max_tokens,
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

    /// Get max tokens.
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    /// Set max tokens.
    pub fn set_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set max tokens (builder pattern).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
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

/// Configuration for Claude Messages Agent (Optional).
///
/// Claude Messages 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct MessagesOptionalConfig {
    metadata: Option<oxide_llm_proto::claude::v1::messages::request::Metadata>,
    stop_sequences: Option<Vec<StaticRefStr>>,
    temperature: Option<f32>,
    tool_choice: Option<ToolChoice>,
    top_k: Option<u32>,
    top_p: Option<f32>,
    thinking: Option<ThinkingConfig>,
    output_config: Option<OutputConfig>,
    service_tier: Option<StaticRefStr>,
}

impl MessagesOptionalConfig {
    /// Create a new `MessagesOptionalConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get metadata.
    pub fn metadata(&self) -> Option<&oxide_llm_proto::claude::v1::messages::request::Metadata> {
        self.metadata.as_ref()
    }

    /// Set metadata.
    pub fn set_metadata(
        &mut self,
        metadata: Option<oxide_llm_proto::claude::v1::messages::request::Metadata>,
    ) -> &mut Self {
        self.metadata = metadata;
        self
    }

    /// Set metadata (builder pattern).
    pub fn with_metadata(
        mut self,
        metadata: oxide_llm_proto::claude::v1::messages::request::Metadata,
    ) -> Self {
        self.metadata = Some(metadata);
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

    /// Get tool choice.
    pub fn tool_choice(&self) -> Option<&ToolChoice> {
        self.tool_choice.as_ref()
    }

    /// Set tool choice.
    pub fn set_tool_choice(&mut self, tool_choice: Option<ToolChoice>) -> &mut Self {
        self.tool_choice = tool_choice;
        self
    }

    /// Set tool choice (builder pattern).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Get top_k.
    pub fn top_k(&self) -> Option<u32> {
        self.top_k
    }

    /// Set top_k.
    pub fn set_top_k(&mut self, top_k: Option<u32>) -> &mut Self {
        self.top_k = top_k;
        self
    }

    /// Set top_k (builder pattern).
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
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

    /// Get thinking config.
    pub fn thinking(&self) -> Option<&ThinkingConfig> {
        self.thinking.as_ref()
    }

    /// Set thinking config.
    pub fn set_thinking(&mut self, thinking: Option<ThinkingConfig>) -> &mut Self {
        self.thinking = thinking;
        self
    }

    /// Set thinking config (builder pattern).
    pub fn with_thinking(mut self, thinking: ThinkingConfig) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Get output config.
    pub fn output_config(&self) -> Option<&OutputConfig> {
        self.output_config.as_ref()
    }

    /// Set output config.
    pub fn set_output_config(&mut self, output_config: Option<OutputConfig>) -> &mut Self {
        self.output_config = output_config;
        self
    }

    /// Set output config (builder pattern).
    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = Some(output_config);
        self
    }

    /// Get service tier.
    pub fn service_tier(&self) -> Option<&str> {
        self.service_tier.as_deref()
    }

    /// Set service tier.
    pub fn set_service_tier(&mut self, service_tier: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.service_tier = service_tier.map(Into::into);
        self
    }

    /// Set service tier (builder pattern).
    pub fn with_service_tier(mut self, service_tier: impl Into<StaticRefStr>) -> Self {
        self.service_tier = Some(service_tier.into());
        self
    }
}

/// Configuration for Claude Messages Agent.
///
/// Claude Messages 代理配置。
#[derive(Debug, Clone)]
pub struct MessagesConfig {
    required: MessagesRequiredConfig,
    optional: MessagesOptionalConfig,
}

impl MessagesConfig {
    /// Create a new `MessagesConfig`.
    pub fn new(required: MessagesRequiredConfig) -> Self {
        Self {
            required,
            optional: MessagesOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    pub fn required(&self) -> &MessagesRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    pub fn required_mut(&mut self) -> &mut MessagesRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    pub fn optional(&self) -> &MessagesOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    pub fn optional_mut(&mut self) -> &mut MessagesOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    pub fn with_optional(mut self, optional: MessagesOptionalConfig) -> Self {
        self.optional = optional;
        self
    }

    /// Convert Config to MessagesRequest with provided messages.
    ///
    /// 将配置转换为 MessagesRequest，并填入消息。
    pub fn to_request(
        self,
        messages: Vec<ClaudeMessage>,
        system: Option<SystemPrompt>,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        stream: bool,
    ) -> MessagesRequest {
        MessagesRequest {
            model: self.required.model.into(),
            messages,
            max_tokens: Some(self.required.max_tokens),
            system,
            metadata: self.optional.metadata,
            stop_sequences: self.optional.stop_sequences,
            stream: Some(stream),
            temperature: self.optional.temperature,
            tool_choice,
            tools,
            top_k: self.optional.top_k,
            top_p: self.optional.top_p,
            thinking: self.optional.thinking,
            output_config: self.optional.output_config,
            service_tier: self.optional.service_tier,
        }
    }
}
