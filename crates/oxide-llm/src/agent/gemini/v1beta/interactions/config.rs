use std::collections::HashMap;

use crate::{
    config::{Config, ReasoningEffort},
    error::AgentError,
};
use oxide_llm_proto::gemini::v1beta::interactions::{
    agent::AgentConfig,
    request::{
        CreateInteractionRequest, GenerationConfig, InteractionsInput, SafetySetting, ServiceTier,
        SpeechConfig, ThinkingLevel, ThinkingSummaries, ToolChoice as GeminiRequestToolChoice,
        TranscriptionConfig, VideoConfig,
    },
    tool::Tool,
    webhook::WebhookConfig,
};
use ref_str::StaticRefStr;
use serde_json::Value;

/// Configuration for Gemini Interactions Agent.
///
/// Gemini Interactions 代理配置。
#[derive(Debug, Clone, Default)]
pub struct InteractionsConfig {
    model: Option<StaticRefStr>,
    agent: Option<StaticRefStr>,
    endpoint: Option<StaticRefStr>,
    system_instruction: Option<String>,
    response_format: Option<Value>,
    stream: Option<bool>,
    store: Option<bool>,
    background: Option<bool>,

    // Generation Config fields
    max_output_tokens: Option<i32>,
    seed: Option<i32>,
    speech_config: Option<SpeechConfig>,
    stop_sequences: Option<Vec<StaticRefStr>>,
    thinking_level: Option<ThinkingLevel>,
    thinking_summaries: Option<ThinkingSummaries>,
    tool_choice: Option<GeminiRequestToolChoice>,
    transcription_config: Option<TranscriptionConfig>,
    video_config: Option<VideoConfig>,

    agent_config: Option<AgentConfig>,
    environment: Option<Value>,
    labels: Option<HashMap<String, String>>,
    previous_interaction_id: Option<String>,
    safety_settings: Option<Vec<SafetySetting>>,
    service_tier: Option<ServiceTier>,
    webhook_config: Option<WebhookConfig>,
}

impl InteractionsConfig {
    /// Create a new `InteractionsConfig`.
    ///
    /// 创建一个新的 `InteractionsConfig`。
    pub fn new() -> Self {
        Self::default()
    }

    /// Get model name.
    ///
    /// 获取模型名称。
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Get model name as StaticRefStr.
    ///
    /// 获取 StaticRefStr 格式的模型名称。
    pub fn model_static(&self) -> Option<StaticRefStr> {
        self.model.clone()
    }

    /// Set model name.
    ///
    /// 设置模型名称。
    pub fn set_model(&mut self, model: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.model = model.map(Into::into);
        self
    }

    /// Set model name (builder pattern).
    ///
    /// 设置模型名称（构建器模式）。
    pub fn with_model(mut self, model: impl Into<StaticRefStr>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Get agent name.
    ///
    /// 获取 Agent 名称。
    pub fn agent(&self) -> Option<&str> {
        self.agent.as_deref()
    }

    /// Get agent name as StaticRefStr.
    ///
    /// 获取 StaticRefStr 格式的 Agent 名称。
    pub fn agent_static(&self) -> Option<StaticRefStr> {
        self.agent.clone()
    }

    /// Set agent name.
    ///
    /// 设置 Agent 名称。
    pub fn set_agent(&mut self, agent: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.agent = agent.map(Into::into);
        self
    }

    /// Set agent name (builder pattern).
    ///
    /// 设置 Agent 名称（构建器模式）。
    pub fn with_agent(mut self, agent: impl Into<StaticRefStr>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Get endpoint.
    ///
    /// 获取 Endpoint。
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// Get endpoint as `StaticRefStr`.
    ///
    /// 获取 `StaticRefStr` 类型的 Endpoint。
    pub fn endpoint_static(&self) -> Option<StaticRefStr> {
        self.endpoint.clone()
    }

    /// Set endpoint.
    ///
    /// 设置 Endpoint。
    pub fn set_endpoint(&mut self, endpoint: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.endpoint = endpoint.map(Into::into);
        self
    }

    /// Set endpoint (builder pattern).
    ///
    /// 设置 Endpoint（构建器模式）。
    pub fn with_endpoint(mut self, endpoint: impl Into<StaticRefStr>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Get system instruction.
    ///
    /// 获取系统指令。
    pub fn system_instruction(&self) -> Option<&str> {
        self.system_instruction.as_deref()
    }

    /// Set system instruction.
    ///
    /// 设置系统指令。
    pub fn set_system_instruction(&mut self, system_instruction: Option<String>) -> &mut Self {
        self.system_instruction = system_instruction;
        self
    }

    /// Set system instruction (builder pattern).
    ///
    /// 设置系统指令（构建器模式）。
    pub fn with_system_instruction(mut self, system_instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(system_instruction.into());
        self
    }

    /// Get response format.
    ///
    /// 获取响应格式。
    pub fn response_format(&self) -> Option<&Value> {
        self.response_format.as_ref()
    }

    /// Set response format.
    ///
    /// 设置响应格式。
    pub fn set_response_format(&mut self, response_format: Option<Value>) -> &mut Self {
        self.response_format = response_format;
        self
    }

    /// Set response format (builder pattern).
    ///
    /// 设置响应格式（构建器模式）。
    pub fn with_response_format(mut self, response_format: Value) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Get stream flag.
    ///
    /// 获取流式传输标志。
    pub fn stream(&self) -> Option<bool> {
        self.stream
    }

    /// Set stream flag.
    ///
    /// 设置流式传输标志。
    pub fn set_stream(&mut self, stream: Option<bool>) -> &mut Self {
        self.stream = stream;
        self
    }

    /// Set stream flag (builder pattern).
    ///
    /// 设置流式传输标志（构建器模式）。
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Get store flag.
    ///
    /// 获取存储标志。
    pub fn store(&self) -> Option<bool> {
        self.store
    }

    /// Set store flag.
    ///
    /// 设置存储标志。
    pub fn set_store(&mut self, store: Option<bool>) -> &mut Self {
        self.store = store;
        self
    }

    /// Set store flag (builder pattern).
    ///
    /// 设置存储标志（构建器模式）。
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Get background flag.
    ///
    /// 获取后台标志。
    pub fn background(&self) -> Option<bool> {
        self.background
    }

    /// Set background flag.
    ///
    /// 设置后台标志。
    pub fn set_background(&mut self, background: Option<bool>) -> &mut Self {
        self.background = background;
        self
    }

    /// Set background flag (builder pattern).
    ///
    /// 设置后台标志（构建器模式）。
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }

    /// Get max output tokens.
    ///
    /// 获取最大输出 Token 数。
    pub fn max_output_tokens(&self) -> Option<i32> {
        self.max_output_tokens
    }

    /// Set max output tokens.
    ///
    /// 设置最大输出 Token 数。
    pub fn set_max_output_tokens(&mut self, max_output_tokens: Option<i32>) -> &mut Self {
        self.max_output_tokens = max_output_tokens;
        self
    }

    /// Set max output tokens (builder pattern).
    ///
    /// 设置最大输出 Token 数（构建器模式）。
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Get seed.
    ///
    /// 获取解码种子。
    pub fn seed(&self) -> Option<i32> {
        self.seed
    }

    /// Set seed.
    ///
    /// 设置解码种子。
    pub fn set_seed(&mut self, seed: Option<i32>) -> &mut Self {
        self.seed = seed;
        self
    }

    /// Set seed (builder pattern).
    ///
    /// 设置解码种子（构建器模式）。
    pub fn with_seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Get speech config.
    ///
    /// 获取语音生成配置。
    pub fn speech_config(&self) -> Option<&SpeechConfig> {
        self.speech_config.as_ref()
    }

    /// Set speech config.
    ///
    /// 设置语音生成配置。
    pub fn set_speech_config(&mut self, speech_config: Option<SpeechConfig>) -> &mut Self {
        self.speech_config = speech_config;
        self
    }

    /// Set speech config (builder pattern).
    ///
    /// 设置语音生成配置（构建器模式）。
    pub fn with_speech_config(mut self, speech_config: SpeechConfig) -> Self {
        self.speech_config = Some(speech_config);
        self
    }

    /// Get stop sequences.
    ///
    /// 获取停止字符序列。
    pub fn stop_sequences(&self) -> Option<&[StaticRefStr]> {
        self.stop_sequences.as_deref()
    }

    /// Set stop sequences.
    ///
    /// 设置停止字符序列。
    pub fn set_stop_sequences(&mut self, stop_sequences: Option<Vec<StaticRefStr>>) -> &mut Self {
        self.stop_sequences = stop_sequences;
        self
    }

    /// Set stop sequences (builder pattern).
    ///
    /// 设置停止字符序列（构建器模式）。
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<StaticRefStr>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Get thinking level.
    ///
    /// 获取思考 Token 级别。
    pub fn thinking_level(&self) -> Option<&ThinkingLevel> {
        self.thinking_level.as_ref()
    }

    /// Set thinking level.
    ///
    /// 设置思考 Token 级别。
    pub fn set_thinking_level(&mut self, thinking_level: Option<ThinkingLevel>) -> &mut Self {
        self.thinking_level = thinking_level;
        self
    }

    /// Set thinking level (builder pattern).
    ///
    /// 设置思考 Token 级别（构建器模式）。
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        self.thinking_level = Some(thinking_level);
        self
    }

    /// Get thinking summaries.
    ///
    /// 获取思考摘要配置。
    pub fn thinking_summaries(&self) -> Option<&ThinkingSummaries> {
        self.thinking_summaries.as_ref()
    }

    /// Set thinking summaries.
    ///
    /// 设置思考摘要配置。
    pub fn set_thinking_summaries(
        &mut self,
        thinking_summaries: Option<ThinkingSummaries>,
    ) -> &mut Self {
        self.thinking_summaries = thinking_summaries;
        self
    }

    /// Set thinking summaries (builder pattern).
    ///
    /// 设置思考摘要配置（构建器模式）。
    pub fn with_thinking_summaries(mut self, thinking_summaries: ThinkingSummaries) -> Self {
        self.thinking_summaries = Some(thinking_summaries);
        self
    }

    /// Get tool choice.
    ///
    /// 获取工具选择配置。
    pub fn tool_choice(&self) -> Option<&GeminiRequestToolChoice> {
        self.tool_choice.as_ref()
    }

    /// Set tool choice.
    ///
    /// 设置工具选择配置。
    pub fn set_tool_choice(&mut self, tool_choice: Option<GeminiRequestToolChoice>) -> &mut Self {
        self.tool_choice = tool_choice;
        self
    }

    /// Set tool choice (builder pattern).
    ///
    /// 设置工具选择配置（构建器模式）。
    pub fn with_tool_choice(mut self, tool_choice: GeminiRequestToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Get transcription config.
    ///
    /// 获取语音识别（转录）配置。
    pub fn transcription_config(&self) -> Option<&TranscriptionConfig> {
        self.transcription_config.as_ref()
    }

    /// Set transcription config.
    ///
    /// 设置语音识别（转录）配置。
    pub fn set_transcription_config(
        &mut self,
        transcription_config: Option<TranscriptionConfig>,
    ) -> &mut Self {
        self.transcription_config = transcription_config;
        self
    }

    /// Set transcription config (builder pattern).
    ///
    /// 设置语音识别（转录）配置（构建器模式）。
    pub fn with_transcription_config(mut self, transcription_config: TranscriptionConfig) -> Self {
        self.transcription_config = Some(transcription_config);
        self
    }

    /// Get video config.
    ///
    /// 获取视频生成配置。
    pub fn video_config(&self) -> Option<&VideoConfig> {
        self.video_config.as_ref()
    }

    /// Set video config.
    ///
    /// 设置视频生成配置。
    pub fn set_video_config(&mut self, video_config: Option<VideoConfig>) -> &mut Self {
        self.video_config = video_config;
        self
    }

    /// Set video config (builder pattern).
    ///
    /// 设置视频生成配置（构建器模式）。
    pub fn with_video_config(mut self, video_config: VideoConfig) -> Self {
        self.video_config = Some(video_config);
        self
    }

    /// Get agent config.
    ///
    /// 获取 Agent 配置。
    pub fn agent_config(&self) -> Option<&AgentConfig> {
        self.agent_config.as_ref()
    }

    /// Set agent config.
    ///
    /// 设置 Agent 配置。
    pub fn set_agent_config(&mut self, agent_config: Option<AgentConfig>) -> &mut Self {
        self.agent_config = agent_config;
        self
    }

    /// Set agent config (builder pattern).
    ///
    /// 设置 Agent 配置（构建器模式）。
    pub fn with_agent_config(mut self, agent_config: AgentConfig) -> Self {
        self.agent_config = Some(agent_config);
        self
    }

    /// Get environment.
    ///
    /// 获取环境参数。
    pub fn environment(&self) -> Option<&Value> {
        self.environment.as_ref()
    }

    /// Set environment.
    ///
    /// 设置环境参数。
    pub fn set_environment(&mut self, environment: Option<Value>) -> &mut Self {
        self.environment = environment;
        self
    }

    /// Set environment (builder pattern).
    ///
    /// 设置环境参数（构建器模式）。
    pub fn with_environment(mut self, environment: Value) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Get labels.
    ///
    /// 获取标签。
    pub fn labels(&self) -> Option<&HashMap<String, String>> {
        self.labels.as_ref()
    }

    /// Set labels.
    ///
    /// 设置标签。
    pub fn set_labels(&mut self, labels: Option<HashMap<String, String>>) -> &mut Self {
        self.labels = labels;
        self
    }

    /// Set labels (builder pattern).
    ///
    /// 设置标签（构建器模式）。
    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Get previous interaction id.
    ///
    /// 获取上一次交互 ID。
    pub fn previous_interaction_id(&self) -> Option<&str> {
        self.previous_interaction_id.as_deref()
    }

    /// Set previous interaction id.
    ///
    /// 设置上一次交互 ID。
    pub fn set_previous_interaction_id(
        &mut self,
        previous_interaction_id: Option<String>,
    ) -> &mut Self {
        self.previous_interaction_id = previous_interaction_id;
        self
    }

    /// Set previous interaction id (builder pattern).
    ///
    /// 设置上一次交互 ID（构建器模式）。
    pub fn with_previous_interaction_id(
        mut self,
        previous_interaction_id: impl Into<String>,
    ) -> Self {
        self.previous_interaction_id = Some(previous_interaction_id.into());
        self
    }

    /// Get safety settings.
    ///
    /// 获取安全设置。
    pub fn safety_settings(&self) -> Option<&[SafetySetting]> {
        self.safety_settings.as_deref()
    }

    /// Set safety settings.
    ///
    /// 设置安全设置。
    pub fn set_safety_settings(
        &mut self,
        safety_settings: Option<Vec<SafetySetting>>,
    ) -> &mut Self {
        self.safety_settings = safety_settings;
        self
    }

    /// Set safety settings (builder pattern).
    ///
    /// 设置安全设置（构建器模式）。
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Get service tier.
    ///
    /// 获取服务层级。
    pub fn service_tier(&self) -> Option<&ServiceTier> {
        self.service_tier.as_ref()
    }

    /// Set service tier.
    ///
    /// 设置服务层级。
    pub fn set_service_tier(&mut self, service_tier: Option<ServiceTier>) -> &mut Self {
        self.service_tier = service_tier;
        self
    }

    /// Set service tier (builder pattern).
    ///
    /// 设置服务层级（构建器模式）。
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Get webhook config.
    ///
    /// 获取 Webhook 配置。
    pub fn webhook_config(&self) -> Option<&WebhookConfig> {
        self.webhook_config.as_ref()
    }

    /// Set webhook config.
    ///
    /// 设置 Webhook 配置。
    pub fn set_webhook_config(&mut self, webhook_config: Option<WebhookConfig>) -> &mut Self {
        self.webhook_config = webhook_config;
        self
    }

    /// Set webhook config (builder pattern).
    ///
    /// 设置 Webhook 配置（构建器模式）。
    pub fn with_webhook_config(mut self, webhook_config: WebhookConfig) -> Self {
        self.webhook_config = Some(webhook_config);
        self
    }

    /// Convert Config to CreateInteractionRequest with provided input, tools, tool_choice, and overrides.
    ///
    /// 将配置转换为 CreateInteractionRequest，并整合输入、工具、工具选择及覆盖项。
    pub fn to_request(
        &self,
        input: InteractionsInput,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<GeminiRequestToolChoice>,
        stream_override: Option<bool>,
        sys_prompt_override: Option<&str>,
    ) -> CreateInteractionRequest {
        let system_instruction = sys_prompt_override
            .map(|s| s.to_string())
            .or_else(|| self.system_instruction.clone());

        let effective_tool_choice = tool_choice.or_else(|| self.tool_choice.clone());

        let generation_config = if self.max_output_tokens.is_some()
            || self.seed.is_some()
            || self.speech_config.is_some()
            || self.stop_sequences.is_some()
            || self.thinking_level.is_some()
            || self.thinking_summaries.is_some()
            || effective_tool_choice.is_some()
            || self.transcription_config.is_some()
            || self.video_config.is_some()
        {
            Some(GenerationConfig {
                max_output_tokens: self.max_output_tokens,
                seed: self.seed,
                speech_config: self.speech_config.clone(),
                stop_sequences: self.stop_sequences.clone(),
                thinking_level: self.thinking_level.clone(),
                thinking_summaries: self.thinking_summaries.clone(),
                tool_choice: effective_tool_choice,
                transcription_config: self.transcription_config.clone(),
                video_config: self.video_config.clone(),
            })
        } else {
            None
        };

        CreateInteractionRequest {
            model: self.model.clone(),
            agent: self.agent.clone(),
            input,
            system_instruction,
            tools,
            response_format: self.response_format.clone(),
            stream: stream_override.or(self.stream),
            store: self.store,
            background: self.background,
            generation_config,
            agent_config: self.agent_config.clone(),
            environment: self.environment.clone(),
            labels: self.labels.clone(),
            previous_interaction_id: self.previous_interaction_id.clone(),
            safety_settings: self.safety_settings.clone(),
            service_tier: self.service_tier.clone(),
            webhook_config: self.webhook_config.clone(),
        }
    }
}

impl TryFrom<Config> for InteractionsConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let mut int_config = Self::new();
        int_config.set_model(Some(config.model_static()));
        if let Some(ep) = config.endpoint_static() {
            int_config.set_endpoint(Some(ep));
        }
        if let Some(stop) = config.stop_sequences() {
            int_config.set_stop_sequences(Some(stop.to_vec()));
        }
        if let Some(seed) = config.seed() {
            int_config.set_seed(Some(seed as i32));
        }
        if let Some(effort) = config.reasoning_effort() {
            let level = match effort {
                ReasoningEffort::None => ThinkingLevel::Minimal,
                ReasoningEffort::Minimal => ThinkingLevel::Minimal,
                ReasoningEffort::Low => ThinkingLevel::Low,
                ReasoningEffort::Medium => ThinkingLevel::Medium,
                ReasoningEffort::High => ThinkingLevel::High,
                ReasoningEffort::Xhigh => ThinkingLevel::High,
                ReasoningEffort::Max => ThinkingLevel::High,
            };
            int_config.set_thinking_level(Some(level));
        }
        if let Some(max_tokens) = config.max_tokens() {
            int_config.set_max_output_tokens(Some(max_tokens as i32));
        }

        Ok(int_config)
    }
}
