use std::collections::HashMap;

use crate::{
    config::{Config, ReasoningEffort as ConfigReasoningEffort, ThinkingConfig},
    error::AgentError,
};
use oxide_llm_proto::openai::v1::response::{
    Tool, ToolChoice,
    config::{ConversationParam, ReasoningConf, ReasoningEffort},
    request::{CreateResponseRequest, InputParam, ResponseStreamOptions},
};
use ref_str::StaticRefStr;

/// Configuration for OpenAI Responses Agent.
///
/// OpenAI Response 代理配置。
#[derive(Debug, Clone)]
pub struct ResponsesConfig {
    model: StaticRefStr,
    endpoint: Option<StaticRefStr>,
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

impl ResponsesConfig {
    /// Create a new `ResponsesConfig`.
    pub fn new(model: impl Into<StaticRefStr>) -> Self {
        Self {
            model: model.into(),
            endpoint: None,
            include: None,
            parallel_tool_calls: None,
            store: None,
            instructions: None,
            metadata: None,
            top_logprobs: None,
            temperature: None,
            top_p: None,
            user: None,
            safety_identifier: None,
            prompt_cache_key: None,
            service_tier: None,
            prompt_cache_retention: None,
            max_output_tokens: None,
            max_tool_calls: None,
            previous_response_id: None,
            stream_options: None,
            conversation: None,
            background: None,
            reasoning: None,
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
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// Get endpoint as `StaticRefStr`.
    pub fn endpoint_static(&self) -> Option<StaticRefStr> {
        self.endpoint.clone()
    }

    /// Set endpoint.
    pub fn set_endpoint(&mut self, endpoint: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.endpoint = endpoint.map(Into::into);
        self
    }

    /// Set endpoint (builder pattern).
    pub fn with_endpoint(mut self, endpoint: impl Into<StaticRefStr>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
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
            model: Some(self.model),
            include: self.include,
            parallel_tool_calls: self.parallel_tool_calls,
            store: self.store,
            instructions: self.instructions,
            stream,
            stream_options: self.stream_options,
            conversation: self.conversation,
            metadata: self.metadata,
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            tools,
            tool_choice,
            user: self.user,
            safety_identifier: self.safety_identifier,
            prompt_cache_key: self.prompt_cache_key,
            service_tier: self.service_tier,
            prompt_cache_retention: self.prompt_cache_retention,
            max_output_tokens: self.max_output_tokens,
            max_tool_calls: self.max_tool_calls,
            previous_response_id: self.previous_response_id,
            reasoning: self.reasoning,
            background: self.background,
            text: None,
            prompt: None,
            truncation: None,
        }
    }
}

impl TryFrom<Config> for ResponsesConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let Config {
            model,
            max_tokens,
            endpoint,
            temperature,
            top_p,
            top_k: _,
            frequency_penalty: _,
            presence_penalty: _,
            stop_sequences: _,
            seed: _,
            reasoning_effort,
            thinking,
        } = config;

        let reasoning_effort = reasoning_effort
            .map(|effort| match effort {
                ConfigReasoningEffort::None => ReasoningEffort::None,
                ConfigReasoningEffort::Minimal => ReasoningEffort::Minimal,
                ConfigReasoningEffort::Low => ReasoningEffort::Low,
                ConfigReasoningEffort::Medium => ReasoningEffort::Medium,
                ConfigReasoningEffort::High => ReasoningEffort::High,
                ConfigReasoningEffort::Xhigh => ReasoningEffort::Xhigh,
                ConfigReasoningEffort::Max => ReasoningEffort::High,
            })
            .or_else(|| {
                thinking.and_then(|t| match t {
                    ThinkingConfig::Bool(true) => Some(ReasoningEffort::Medium),
                    ThinkingConfig::Bool(false) => Some(ReasoningEffort::None),
                    ThinkingConfig::Budget(_) => Some(ReasoningEffort::Medium),
                    ThinkingConfig::Full { enabled, .. } => match enabled {
                        Some(true) => Some(ReasoningEffort::Medium),
                        Some(false) => Some(ReasoningEffort::None),
                        None => None,
                    },
                })
            });

        let reasoning = reasoning_effort.map(|effort| ReasoningConf {
            effort: Some(effort),
            summary: None,
            generate_summary: None,
        });

        Ok(Self {
            model,
            endpoint,
            include: None,
            parallel_tool_calls: None,
            store: None,
            instructions: None,
            metadata: None,
            top_logprobs: None,
            temperature,
            top_p,
            user: None,
            safety_identifier: None,
            prompt_cache_key: None,
            service_tier: None,
            prompt_cache_retention: None,
            max_output_tokens: max_tokens,
            max_tool_calls: None,
            previous_response_id: None,
            stream_options: None,
            conversation: None,
            background: None,
            reasoning,
        })
    }
}
