use std::collections::HashMap;

use oxide_llm_proto::openai::v1::chat_completions::request::{
    AudioOptions, ChatCompletionMessage, ChatCompletionRequest, PredictionContent, ResponseFormat,
    Stop, StreamOptions, WebSearchOptions,
};
use oxide_llm_proto::openai::v1::chat_completions::{
    FunctionDefinition, ReasoningEffort, Tool, ToolChoice,
};
use ref_str::StaticRefStr;

/// Configuration for OpenAI Chat Completions Agent (Required).
///
/// OpenAI Chat Completions 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct ChatCompletionsRequiredConfig {
    model: StaticRefStr,
    endpoint: StaticRefStr,
}

impl ChatCompletionsRequiredConfig {
    /// Create a new `ChatCompletionsRequiredConfig`.
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

/// Configuration for OpenAI Chat Completions Agent (Optional).
///
/// OpenAI Chat Completions 代理配置 (选填)。
/// 包含了除 `messages` 和 `model` 之外的所有 `ChatCompletionRequest` 可选参数。
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionsOptionalConfig {
    frequency_penalty: Option<f32>,
    logit_bias: Option<HashMap<StaticRefStr, f32>>,
    logprobs: Option<bool>,
    top_logprobs: Option<u8>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    n: Option<u8>,
    modalities: Option<Vec<StaticRefStr>>,
    prediction: Option<PredictionContent>,
    audio: Option<AudioOptions>,
    presence_penalty: Option<f32>,
    response_format: Option<ResponseFormat>,
    seed: Option<i64>,
    service_tier: Option<StaticRefStr>,
    stop: Option<Stop>,
    store: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    parallel_tool_calls: Option<bool>,
    user: Option<StaticRefStr>,
    function_call: Option<serde_json::Value>,
    functions: Option<Vec<FunctionDefinition>>,
    web_search_options: Option<WebSearchOptions>,
    verbosity: Option<StaticRefStr>,
    reasoning_effort: Option<ReasoningEffort>,
    metadata: Option<HashMap<StaticRefStr, StaticRefStr>>,
}

impl ChatCompletionsOptionalConfig {
    /// Create a new `ChatCompletionsOptionalConfig`.
    pub fn new() -> Self {
        Self::default()
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

    /// Get logit bias.
    pub fn logit_bias(&self) -> Option<&HashMap<StaticRefStr, f32>> {
        self.logit_bias.as_ref()
    }

    /// Set logit bias.
    pub fn set_logit_bias(&mut self, logit_bias: Option<HashMap<StaticRefStr, f32>>) -> &mut Self {
        self.logit_bias = logit_bias;
        self
    }

    /// Set logit bias (builder pattern).
    pub fn with_logit_bias(mut self, logit_bias: HashMap<StaticRefStr, f32>) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    /// Get logprobs.
    pub fn logprobs(&self) -> Option<bool> {
        self.logprobs
    }

    /// Set logprobs.
    pub fn set_logprobs(&mut self, logprobs: Option<bool>) -> &mut Self {
        self.logprobs = logprobs;
        self
    }

    /// Set logprobs (builder pattern).
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
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

    /// Get max tokens.
    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    /// Set max tokens.
    pub fn set_max_tokens(&mut self, max_tokens: Option<u32>) -> &mut Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set max tokens (builder pattern).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Get max completion tokens.
    pub fn max_completion_tokens(&self) -> Option<u32> {
        self.max_completion_tokens
    }

    /// Set max completion tokens.
    pub fn set_max_completion_tokens(&mut self, max_completion_tokens: Option<u32>) -> &mut Self {
        self.max_completion_tokens = max_completion_tokens;
        self
    }

    /// Set max completion tokens (builder pattern).
    pub fn with_max_completion_tokens(mut self, max_completion_tokens: u32) -> Self {
        self.max_completion_tokens = Some(max_completion_tokens);
        self
    }

    /// Get n.
    pub fn n(&self) -> Option<u8> {
        self.n
    }

    /// Set n.
    pub fn set_n(&mut self, n: Option<u8>) -> &mut Self {
        self.n = n;
        self
    }

    /// Set n (builder pattern).
    pub fn with_n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// Get modalities.
    pub fn modalities(&self) -> Option<&[StaticRefStr]> {
        self.modalities.as_deref()
    }

    /// Set modalities.
    pub fn set_modalities(&mut self, modalities: Option<Vec<StaticRefStr>>) -> &mut Self {
        self.modalities = modalities;
        self
    }

    /// Set modalities (builder pattern).
    pub fn with_modalities(mut self, modalities: Vec<StaticRefStr>) -> Self {
        self.modalities = Some(modalities);
        self
    }

    /// Get prediction.
    pub fn prediction(&self) -> Option<&PredictionContent> {
        self.prediction.as_ref()
    }

    /// Set prediction.
    pub fn set_prediction(&mut self, prediction: Option<PredictionContent>) -> &mut Self {
        self.prediction = prediction;
        self
    }

    /// Set prediction (builder pattern).
    pub fn with_prediction(mut self, prediction: PredictionContent) -> Self {
        self.prediction = Some(prediction);
        self
    }

    /// Get audio.
    pub fn audio(&self) -> Option<&AudioOptions> {
        self.audio.as_ref()
    }

    /// Set audio.
    pub fn set_audio(&mut self, audio: Option<AudioOptions>) -> &mut Self {
        self.audio = audio;
        self
    }

    /// Set audio (builder pattern).
    pub fn with_audio(mut self, audio: AudioOptions) -> Self {
        self.audio = Some(audio);
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

    /// Get response format.
    pub fn response_format(&self) -> Option<&ResponseFormat> {
        self.response_format.as_ref()
    }

    /// Set response format.
    pub fn set_response_format(&mut self, response_format: Option<ResponseFormat>) -> &mut Self {
        self.response_format = response_format;
        self
    }

    /// Set response format (builder pattern).
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Get seed.
    pub fn seed(&self) -> Option<i64> {
        self.seed
    }

    /// Set seed.
    pub fn set_seed(&mut self, seed: Option<i64>) -> &mut Self {
        self.seed = seed;
        self
    }

    /// Set seed (builder pattern).
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
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

    /// Get stop.
    pub fn stop(&self) -> Option<&Stop> {
        self.stop.as_ref()
    }

    /// Set stop.
    pub fn set_stop(&mut self, stop: Option<Stop>) -> &mut Self {
        self.stop = stop;
        self
    }

    /// Set stop (builder pattern).
    pub fn with_stop(mut self, stop: Stop) -> Self {
        self.stop = Some(stop);
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

    /// Get parallel tool calls.
    pub fn parallel_tool_calls(&self) -> Option<bool> {
        self.parallel_tool_calls
    }

    /// Set parallel tool calls.
    pub fn set_parallel_tool_calls(&mut self, parallel_tool_calls: Option<bool>) -> &mut Self {
        self.parallel_tool_calls = parallel_tool_calls;
        self
    }

    /// Set parallel tool calls (builder pattern).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
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

    /// Get function call.
    pub fn function_call(&self) -> Option<&serde_json::Value> {
        self.function_call.as_ref()
    }

    /// Set function call.
    pub fn set_function_call(&mut self, function_call: Option<serde_json::Value>) -> &mut Self {
        self.function_call = function_call;
        self
    }

    /// Set function call (builder pattern).
    pub fn with_function_call(mut self, function_call: serde_json::Value) -> Self {
        self.function_call = Some(function_call);
        self
    }

    /// Get functions.
    pub fn functions(&self) -> Option<&[FunctionDefinition]> {
        self.functions.as_deref()
    }

    /// Set functions.
    pub fn set_functions(&mut self, functions: Option<Vec<FunctionDefinition>>) -> &mut Self {
        self.functions = functions;
        self
    }

    /// Set functions (builder pattern).
    pub fn with_functions(mut self, functions: Vec<FunctionDefinition>) -> Self {
        self.functions = Some(functions);
        self
    }

    /// Get web search options.
    pub fn web_search_options(&self) -> Option<&WebSearchOptions> {
        self.web_search_options.as_ref()
    }

    /// Set web search options.
    pub fn set_web_search_options(
        &mut self,
        web_search_options: Option<WebSearchOptions>,
    ) -> &mut Self {
        self.web_search_options = web_search_options;
        self
    }

    /// Set web search options (builder pattern).
    pub fn with_web_search_options(mut self, web_search_options: WebSearchOptions) -> Self {
        self.web_search_options = Some(web_search_options);
        self
    }

    /// Get verbosity.
    pub fn verbosity(&self) -> Option<&str> {
        self.verbosity.as_deref()
    }

    /// Set verbosity.
    pub fn set_verbosity(&mut self, verbosity: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.verbosity = verbosity.map(Into::into);
        self
    }

    /// Set verbosity (builder pattern).
    pub fn with_verbosity(mut self, verbosity: impl Into<StaticRefStr>) -> Self {
        self.verbosity = Some(verbosity.into());
        self
    }

    /// Get reasoning effort.
    pub fn reasoning_effort(&self) -> Option<&ReasoningEffort> {
        self.reasoning_effort.as_ref()
    }

    /// Set reasoning effort.
    pub fn set_reasoning_effort(
        &mut self,
        reasoning_effort: Option<ReasoningEffort>,
    ) -> &mut Self {
        self.reasoning_effort = reasoning_effort;
        self
    }

    /// Set reasoning effort (builder pattern).
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
        self
    }

    /// Get metadata.
    pub fn metadata(&self) -> Option<&HashMap<StaticRefStr, StaticRefStr>> {
        self.metadata.as_ref()
    }

    /// Set metadata.
    pub fn set_metadata(
        &mut self,
        metadata: Option<HashMap<StaticRefStr, StaticRefStr>>,
    ) -> &mut Self {
        self.metadata = metadata;
        self
    }

    /// Set metadata (builder pattern).
    pub fn with_metadata(
        mut self,
        metadata: HashMap<StaticRefStr, StaticRefStr>,
    ) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Configuration for OpenAI Chat Completions Agent.
///
/// OpenAI Chat Completions 代理配置。
#[derive(Debug, Clone)]
pub struct ChatCompletionsConfig {
    required: ChatCompletionsRequiredConfig,
    optional: ChatCompletionsOptionalConfig,
}

impl ChatCompletionsConfig {
    /// Create a new `ChatCompletionsConfig`.
    pub fn new(required: ChatCompletionsRequiredConfig) -> Self {
        Self {
            required,
            optional: ChatCompletionsOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    pub fn required(&self) -> &ChatCompletionsRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    pub fn required_mut(&mut self) -> &mut ChatCompletionsRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    pub fn optional(&self) -> &ChatCompletionsOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    pub fn optional_mut(&mut self) -> &mut ChatCompletionsOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    pub fn with_optional(mut self, optional: ChatCompletionsOptionalConfig) -> Self {
        self.optional = optional;
        self
    }

    /// Convert Config to ChatCompletionRequest with provided messages.
    ///
    /// 将配置转换为 ChatCompletionRequest，并填入消息。
    pub fn to_request(
        self,
        messages: Vec<ChatCompletionMessage>,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        is_stream: bool,
        stream_options: Option<StreamOptions>,
    ) -> ChatCompletionRequest {
        ChatCompletionRequest {
            messages,
            model: self.required.model,
            frequency_penalty: self.optional.frequency_penalty,
            logit_bias: self.optional.logit_bias,
            logprobs: self.optional.logprobs,
            top_logprobs: self.optional.top_logprobs,
            max_tokens: self.optional.max_tokens,
            max_completion_tokens: self.optional.max_completion_tokens,
            n: self.optional.n,
            modalities: self.optional.modalities,
            prediction: self.optional.prediction,
            audio: self.optional.audio,
            presence_penalty: self.optional.presence_penalty,
            response_format: self.optional.response_format,
            seed: self.optional.seed,
            service_tier: self.optional.service_tier,
            stop: self.optional.stop,
            store: self.optional.store,
            stream: Some(is_stream),
            stream_options,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            tools,
            tool_choice,
            parallel_tool_calls: self.optional.parallel_tool_calls,
            user: self.optional.user,
            function_call: self.optional.function_call,
            functions: self.optional.functions,
            web_search_options: self.optional.web_search_options,
            verbosity: self.optional.verbosity,
            reasoning_effort: self.optional.reasoning_effort,
            metadata: self.optional.metadata,
        }
    }
}
