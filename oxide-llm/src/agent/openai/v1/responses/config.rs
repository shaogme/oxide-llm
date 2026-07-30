use std::collections::HashMap;

use crate::{
    config::{Config, OptionalConfig, ReasoningEffort as ConfigReasoningEffort, RequiredConfig},
    error::AgentError,
};
use oxide_llm_proto::openai::v1::response::{
    Tool, ToolChoice,
    config::{ConversationParam, ReasoningConf, ReasoningEffort},
    request::{CreateResponseRequest, InputParam, ResponseStreamOptions},
};
use ref_str::StaticRefStr;

/// Configuration for OpenAI Responses Agent (Required).
///
/// OpenAI Response 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct ResponsesRequiredConfig {
    model: StaticRefStr,
    endpoint: StaticRefStr,
}

impl ResponsesRequiredConfig {
    /// Create a new `ResponsesRequiredConfig`.
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

/// Configuration for OpenAI Responses Agent (Optional).
///
/// OpenAI Response 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct ResponsesOptionalConfig {
    include: Option<Vec<StaticRefStr>>,
    parallel_tool_calls: Option<bool>,
    store: Option<bool>,
    instructions: Option<StaticRefStr>,
    metadata: Option<HashMap<StaticRefStr, serde_json::Value>>,
    top_logprobs: Option<u8>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    user: Option<StaticRefStr>,
    safety_identifier: Option<StaticRefStr>,
    prompt_cache_key: Option<StaticRefStr>,
    service_tier: Option<StaticRefStr>,
    prompt_cache_retention: Option<StaticRefStr>,
    max_output_tokens: Option<u32>,
    max_tool_calls: Option<u32>,
    previous_response_id: Option<StaticRefStr>,
    stream_options: Option<ResponseStreamOptions>,
    conversation: Option<ConversationParam>,
    background: Option<bool>,
    reasoning: Option<ReasoningConf>,
}

impl ResponsesOptionalConfig {
    /// Create a new `ResponsesOptionalConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get include list.
    pub fn include(&self) -> Option<&[StaticRefStr]> {
        self.include.as_deref()
    }

    /// Set include list.
    pub fn set_include(&mut self, include: Option<Vec<StaticRefStr>>) -> &mut Self {
        self.include = include;
        self
    }

    /// Set include list (builder pattern).
    pub fn with_include(mut self, include: Vec<StaticRefStr>) -> Self {
        self.include = Some(include);
        self
    }

    /// Get parallel tool calls option.
    pub fn parallel_tool_calls(&self) -> Option<bool> {
        self.parallel_tool_calls
    }

    /// Set parallel tool calls option.
    pub fn set_parallel_tool_calls(&mut self, parallel_tool_calls: Option<bool>) -> &mut Self {
        self.parallel_tool_calls = parallel_tool_calls;
        self
    }

    /// Set parallel tool calls option (builder pattern).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Get store option.
    pub fn store(&self) -> Option<bool> {
        self.store
    }

    /// Set store option.
    pub fn set_store(&mut self, store: Option<bool>) -> &mut Self {
        self.store = store;
        self
    }

    /// Set store option (builder pattern).
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Get instructions.
    pub fn instructions(&self) -> Option<&str> {
        self.instructions.as_deref()
    }

    /// Set instructions.
    pub fn set_instructions(&mut self, instructions: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.instructions = instructions.map(Into::into);
        self
    }

    /// Set instructions (builder pattern).
    pub fn with_instructions(mut self, instructions: impl Into<StaticRefStr>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Get metadata.
    pub fn metadata(&self) -> Option<&HashMap<StaticRefStr, serde_json::Value>> {
        self.metadata.as_ref()
    }

    /// Set metadata.
    pub fn set_metadata(
        &mut self,
        metadata: Option<HashMap<StaticRefStr, serde_json::Value>>,
    ) -> &mut Self {
        self.metadata = metadata;
        self
    }

    /// Set metadata (builder pattern).
    pub fn with_metadata(mut self, metadata: HashMap<StaticRefStr, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get top logprobs.
    pub fn top_logprobs(&self) -> Option<u8> {
        self.top_logprobs
    }

    /// Set top logprobs.
    pub fn set_top_logprobs(&mut self, top_logprobs: Option<u8>) -> &mut Self {
        self.top_logprobs = top_logprobs;
        self
    }

    /// Set top logprobs (builder pattern).
    pub fn with_top_logprobs(mut self, top_logprobs: u8) -> Self {
        self.top_logprobs = Some(top_logprobs);
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

    /// Get user.
    pub fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// Set user.
    pub fn set_user(&mut self, user: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.user = user.map(Into::into);
        self
    }

    /// Set user (builder pattern).
    pub fn with_user(mut self, user: impl Into<StaticRefStr>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Get safety identifier.
    pub fn safety_identifier(&self) -> Option<&str> {
        self.safety_identifier.as_deref()
    }

    /// Set safety identifier.
    pub fn set_safety_identifier(
        &mut self,
        safety_identifier: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.safety_identifier = safety_identifier.map(Into::into);
        self
    }

    /// Set safety identifier (builder pattern).
    pub fn with_safety_identifier(mut self, safety_identifier: impl Into<StaticRefStr>) -> Self {
        self.safety_identifier = Some(safety_identifier.into());
        self
    }

    /// Get prompt cache key.
    pub fn prompt_cache_key(&self) -> Option<&str> {
        self.prompt_cache_key.as_deref()
    }

    /// Set prompt cache key.
    pub fn set_prompt_cache_key(
        &mut self,
        prompt_cache_key: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.prompt_cache_key = prompt_cache_key.map(Into::into);
        self
    }

    /// Set prompt cache key (builder pattern).
    pub fn with_prompt_cache_key(mut self, prompt_cache_key: impl Into<StaticRefStr>) -> Self {
        self.prompt_cache_key = Some(prompt_cache_key.into());
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

    /// Get prompt cache retention.
    pub fn prompt_cache_retention(&self) -> Option<&str> {
        self.prompt_cache_retention.as_deref()
    }

    /// Set prompt cache retention.
    pub fn set_prompt_cache_retention(
        &mut self,
        prompt_cache_retention: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.prompt_cache_retention = prompt_cache_retention.map(Into::into);
        self
    }

    /// Set prompt cache retention (builder pattern).
    pub fn with_prompt_cache_retention(
        mut self,
        prompt_cache_retention: impl Into<StaticRefStr>,
    ) -> Self {
        self.prompt_cache_retention = Some(prompt_cache_retention.into());
        self
    }

    /// Get max output tokens.
    pub fn max_output_tokens(&self) -> Option<u32> {
        self.max_output_tokens
    }

    /// Set max output tokens.
    pub fn set_max_output_tokens(&mut self, max_output_tokens: Option<u32>) -> &mut Self {
        self.max_output_tokens = max_output_tokens;
        self
    }

    /// Set max output tokens (builder pattern).
    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Get max tool calls.
    pub fn max_tool_calls(&self) -> Option<u32> {
        self.max_tool_calls
    }

    /// Set max tool calls.
    pub fn set_max_tool_calls(&mut self, max_tool_calls: Option<u32>) -> &mut Self {
        self.max_tool_calls = max_tool_calls;
        self
    }

    /// Set max tool calls (builder pattern).
    pub fn with_max_tool_calls(mut self, max_tool_calls: u32) -> Self {
        self.max_tool_calls = Some(max_tool_calls);
        self
    }

    /// Get previous response id.
    pub fn previous_response_id(&self) -> Option<&str> {
        self.previous_response_id.as_deref()
    }

    /// Set previous response id.
    pub fn set_previous_response_id(
        &mut self,
        previous_response_id: Option<impl Into<StaticRefStr>>,
    ) -> &mut Self {
        self.previous_response_id = previous_response_id.map(Into::into);
        self
    }

    /// Set previous response id (builder pattern).
    pub fn with_previous_response_id(
        mut self,
        previous_response_id: impl Into<StaticRefStr>,
    ) -> Self {
        self.previous_response_id = Some(previous_response_id.into());
        self
    }

    /// Get stream options.
    pub fn stream_options(&self) -> Option<&ResponseStreamOptions> {
        self.stream_options.as_ref()
    }

    /// Set stream options.
    pub fn set_stream_options(
        &mut self,
        stream_options: Option<ResponseStreamOptions>,
    ) -> &mut Self {
        self.stream_options = stream_options;
        self
    }

    /// Set stream options (builder pattern).
    pub fn with_stream_options(mut self, stream_options: ResponseStreamOptions) -> Self {
        self.stream_options = Some(stream_options);
        self
    }

    /// Get conversation parameter.
    pub fn conversation(&self) -> Option<&ConversationParam> {
        self.conversation.as_ref()
    }

    /// Set conversation parameter.
    pub fn set_conversation(&mut self, conversation: Option<ConversationParam>) -> &mut Self {
        self.conversation = conversation;
        self
    }

    /// Set conversation parameter (builder pattern).
    pub fn with_conversation(mut self, conversation: ConversationParam) -> Self {
        self.conversation = Some(conversation);
        self
    }

    /// Get background option.
    pub fn background(&self) -> Option<bool> {
        self.background
    }

    /// Set background option.
    pub fn set_background(&mut self, background: Option<bool>) -> &mut Self {
        self.background = background;
        self
    }

    /// Set background option (builder pattern).
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }

    /// Get reasoning config.
    pub fn reasoning(&self) -> Option<&ReasoningConf> {
        self.reasoning.as_ref()
    }

    /// Set reasoning config.
    pub fn set_reasoning(&mut self, reasoning: Option<ReasoningConf>) -> &mut Self {
        self.reasoning = reasoning;
        self
    }

    /// Set reasoning config (builder pattern).
    pub fn with_reasoning(mut self, reasoning: ReasoningConf) -> Self {
        self.reasoning = Some(reasoning);
        self
    }
}

/// Configuration for OpenAI Responses Agent.
///
/// OpenAI Response 代理配置。
#[derive(Debug, Clone)]
pub struct ResponsesConfig {
    required: ResponsesRequiredConfig,
    optional: ResponsesOptionalConfig,
}

impl ResponsesConfig {
    /// Create a new `ResponsesConfig`.
    pub fn new(required: ResponsesRequiredConfig) -> Self {
        Self {
            required,
            optional: ResponsesOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    pub fn required(&self) -> &ResponsesRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    pub fn required_mut(&mut self) -> &mut ResponsesRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    pub fn optional(&self) -> &ResponsesOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    pub fn optional_mut(&mut self) -> &mut ResponsesOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    pub fn with_optional(mut self, optional: ResponsesOptionalConfig) -> Self {
        self.optional = optional;
        self
    }

    /// Convert Config to CreateResponseRequest with provided input.
    ///
    /// 将配置转换为 CreateResponseRequest，并填入输入参数。
    pub fn to_request(
        self,
        input: InputParam,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        stream: Option<bool>,
    ) -> CreateResponseRequest {
        CreateResponseRequest {
            input,
            model: Some(self.required.model),
            include: self.optional.include,
            parallel_tool_calls: self.optional.parallel_tool_calls,
            store: self.optional.store,
            instructions: self.optional.instructions,
            stream,
            stream_options: self.optional.stream_options,
            conversation: self.optional.conversation,
            metadata: self.optional.metadata,
            top_logprobs: self.optional.top_logprobs,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            tools,
            tool_choice,
            user: self.optional.user,
            safety_identifier: self.optional.safety_identifier,
            prompt_cache_key: self.optional.prompt_cache_key,
            service_tier: self.optional.service_tier,
            prompt_cache_retention: self.optional.prompt_cache_retention,
            max_output_tokens: self.optional.max_output_tokens,
            max_tool_calls: self.optional.max_tool_calls,
            previous_response_id: self.optional.previous_response_id,
            reasoning: self.optional.reasoning,
            background: self.optional.background,
            text: None,
            prompt: None,
            truncation: None,
        }
    }
}

impl crate::agent::builder::AgentConfigTrait for ResponsesConfig {
    type Required = ResponsesRequiredConfig;
    type Optional = ResponsesOptionalConfig;

    fn from_required(required: Self::Required) -> Self {
        Self::new(required)
    }

    fn with_optional(self, optional: Self::Optional) -> Self {
        self.with_optional(optional)
    }
}

impl TryFrom<RequiredConfig> for ResponsesRequiredConfig {
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

impl TryFrom<OptionalConfig> for ResponsesOptionalConfig {
    type Error = AgentError;

    fn try_from(config: OptionalConfig) -> Result<Self, Self::Error> {
        let OptionalConfig {
            temperature,
            top_p,
            top_k: _,
            frequency_penalty: _,
            presence_penalty: _,
            stop_sequences: _,
            seed: _,
            reasoning_effort,
        } = config;

        let mut optional = Self::new();
        if let Some(temp) = temperature {
            optional.set_temperature(Some(temp));
        }
        if let Some(top_p) = top_p {
            optional.set_top_p(Some(top_p));
        }
        if let Some(effort) = reasoning_effort {
            let reasoning_effort = match effort {
                ConfigReasoningEffort::None => ReasoningEffort::None,
                ConfigReasoningEffort::Minimal => ReasoningEffort::Minimal,
                ConfigReasoningEffort::Low => ReasoningEffort::Low,
                ConfigReasoningEffort::Medium => ReasoningEffort::Medium,
                ConfigReasoningEffort::High => ReasoningEffort::High,
                ConfigReasoningEffort::Xhigh => ReasoningEffort::Xhigh,
                ConfigReasoningEffort::Max => ReasoningEffort::High,
            };
            optional.set_reasoning(Some(ReasoningConf {
                effort: Some(reasoning_effort),
                summary: None,
                generate_summary: None,
            }));
        }
        Ok(optional)
    }
}

impl TryFrom<Config> for ResponsesConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let (required, optional) = (config.required().clone(), config.optional().clone());
        let required = ResponsesRequiredConfig::try_from(required)?;
        let optional = ResponsesOptionalConfig::try_from(optional)?;
        if let Some(max_tokens) = config.required().max_tokens() {
            let mut optional = optional;
            optional.set_max_output_tokens(Some(max_tokens));
            Ok(Self::new(required).with_optional(optional))
        } else {
            Ok(Self::new(required).with_optional(optional))
        }
    }
}
