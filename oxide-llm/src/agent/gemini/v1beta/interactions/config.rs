use std::collections::HashMap;

use oxide_llm_proto::gemini::v1beta::interactions::{
    agent::AgentConfig,
    request::{CreateInteractionRequest, GenerationConfig, SafetySetting, ServiceTier},
    webhook::WebhookConfig,
};
use ref_str::StaticRefStr;
use serde_json::Value;

/// Configuration for Gemini Interactions Agent (Required).
///
/// Gemini Interactions 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct InteractionsRequiredConfig {
    model: Option<StaticRefStr>,
    agent: Option<StaticRefStr>,
    endpoint: StaticRefStr,
}

impl InteractionsRequiredConfig {
    /// Create a new `InteractionsRequiredConfig`.
    ///
    /// 创建一个新的 `InteractionsRequiredConfig`。
    pub fn new(endpoint: impl Into<StaticRefStr>) -> Self {
        Self {
            model: None,
            agent: None,
            endpoint: endpoint.into(),
        }
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
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Set endpoint.
    ///
    /// 设置 Endpoint。
    pub fn set_endpoint(&mut self, endpoint: impl Into<StaticRefStr>) -> &mut Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set endpoint (builder pattern).
    ///
    /// 设置 Endpoint（构建器模式）。
    pub fn with_endpoint(mut self, endpoint: impl Into<StaticRefStr>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

/// Configuration for Gemini Interactions Agent (Optional).
///
/// Gemini Interactions 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct InteractionsOptionalConfig {
    system_instruction: Option<String>,
    response_format: Option<Value>,
    store: Option<bool>,
    background: Option<bool>,
    generation_config: Option<GenerationConfig>,
    agent_config: Option<AgentConfig>,
    environment: Option<Value>,
    labels: Option<HashMap<String, String>>,
    previous_interaction_id: Option<String>,
    safety_settings: Option<Vec<SafetySetting>>,
    service_tier: Option<ServiceTier>,
    webhook_config: Option<WebhookConfig>,
}

impl InteractionsOptionalConfig {
    /// Create a new `InteractionsOptionalConfig`.
    ///
    /// 创建一个新的 `InteractionsOptionalConfig`。
    pub fn new() -> Self {
        Self::default()
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

    /// Get generation config.
    ///
    /// 获取生成配置。
    pub fn generation_config(&self) -> Option<&GenerationConfig> {
        self.generation_config.as_ref()
    }

    /// Set generation config.
    ///
    /// 设置生成配置。
    pub fn set_generation_config(
        &mut self,
        generation_config: Option<GenerationConfig>,
    ) -> &mut Self {
        self.generation_config = generation_config;
        self
    }

    /// Set generation config (builder pattern).
    ///
    /// 设置生成配置（构建器模式）。
    pub fn with_generation_config(mut self, generation_config: GenerationConfig) -> Self {
        self.generation_config = Some(generation_config);
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
}

/// Configuration for Gemini Interactions Agent.
///
/// Gemini Interactions 代理配置。
#[derive(Debug, Clone)]
pub struct InteractionsConfig {
    required: InteractionsRequiredConfig,
    optional: InteractionsOptionalConfig,
}

impl InteractionsConfig {
    /// Create a new `InteractionsConfig`.
    ///
    /// 创建一个新的 `InteractionsConfig`。
    pub fn new(required: InteractionsRequiredConfig) -> Self {
        Self {
            required,
            optional: InteractionsOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    ///
    /// 获取必填配置引用。
    pub fn required(&self) -> &InteractionsRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    ///
    /// 获取必填配置的可变引用。
    pub fn required_mut(&mut self) -> &mut InteractionsRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    ///
    /// 获取选填配置引用。
    pub fn optional(&self) -> &InteractionsOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    ///
    /// 获取选填配置的可变引用。
    pub fn optional_mut(&mut self) -> &mut InteractionsOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    ///
    /// 设置选填配置（构建器模式）。
    pub fn with_optional(mut self, optional: InteractionsOptionalConfig) -> Self {
        self.optional = optional;
        self
    }

    /// Merge optional configuration with base request.
    ///
    /// 将选填配置合并到基础请求中。
    pub fn apply_to_request(
        &self,
        mut req: CreateInteractionRequest,
        stream_override: Option<bool>,
        sys_prompt: Option<&str>,
    ) -> CreateInteractionRequest {
        if self.required.agent.is_some() {
            req.agent = self.required.agent.clone();
        }
        if self.required.model.is_some() {
            req.model = self.required.model.clone();
        }

        // Priority for system instruction: state system prompt, then optional config
        let sys_inst = sys_prompt
            .map(|s| s.to_string())
            .or_else(|| self.optional.system_instruction.clone());
        req.system_instruction = sys_inst;

        req.response_format = self.optional.response_format.clone();
        req.stream = stream_override;
        req.store = self.optional.store;
        req.background = self.optional.background;
        req.agent_config = self.optional.agent_config.clone();
        req.environment = self.optional.environment.clone();
        req.labels = self.optional.labels.clone();
        req.previous_interaction_id = self.optional.previous_interaction_id.clone();
        req.safety_settings = self.optional.safety_settings.clone();
        req.service_tier = self.optional.service_tier.clone();
        req.webhook_config = self.optional.webhook_config.clone();

        if let Some(opt_gen) = &self.optional.generation_config {
            let mut merged_gen = req.generation_config.unwrap_or_else(|| GenerationConfig {
                max_output_tokens: None,
                seed: None,
                speech_config: None,
                stop_sequences: None,
                thinking_level: None,
                thinking_summaries: None,
                tool_choice: None,
                transcription_config: None,
                video_config: None,
            });

            if opt_gen.max_output_tokens.is_some() {
                merged_gen.max_output_tokens = opt_gen.max_output_tokens;
            }
            if opt_gen.seed.is_some() {
                merged_gen.seed = opt_gen.seed;
            }
            if opt_gen.speech_config.is_some() {
                merged_gen.speech_config = opt_gen.speech_config.clone();
            }
            if opt_gen.stop_sequences.is_some() {
                merged_gen.stop_sequences = opt_gen.stop_sequences.clone();
            }
            if opt_gen.thinking_level.is_some() {
                merged_gen.thinking_level = opt_gen.thinking_level.clone();
            }
            if opt_gen.thinking_summaries.is_some() {
                merged_gen.thinking_summaries = opt_gen.thinking_summaries.clone();
            }
            if opt_gen.tool_choice.is_some() {
                merged_gen.tool_choice = opt_gen.tool_choice.clone();
            }
            if opt_gen.transcription_config.is_some() {
                merged_gen.transcription_config = opt_gen.transcription_config.clone();
            }
            if opt_gen.video_config.is_some() {
                merged_gen.video_config = opt_gen.video_config.clone();
            }

            req.generation_config = Some(merged_gen);
        }

        req
    }
}
