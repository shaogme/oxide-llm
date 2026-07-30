use crate::{
    config::{Config, OptionalConfig, ReasoningEffort, RequiredConfig},
    error::AgentError,
};
use oxide_llm_proto::claude::v1::messages::{
    Container, Message as ClaudeMessage, MessagesRequest, Metadata, OutputConfig, OutputEffort,
    OutputFormat, SystemPrompt, ThinkingConfigParam, Tool, ToolChoice,
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
    ///
    /// 创建新的 `MessagesRequiredConfig`。
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
    ///
    /// 获取模型名称。
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Set model name.
    ///
    /// 设置模型名称。
    pub fn set_model(&mut self, model: impl Into<StaticRefStr>) -> &mut Self {
        self.model = model.into();
        self
    }

    /// Set model name (builder pattern).
    ///
    /// 设置模型名称（构建器模式）。
    pub fn with_model(mut self, model: impl Into<StaticRefStr>) -> Self {
        self.model = model.into();
        self
    }

    /// Get max tokens.
    ///
    /// 获取最大 token 限制。
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    /// Set max tokens.
    ///
    /// 设置最大 token 限制。
    pub fn set_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set max tokens (builder pattern).
    ///
    /// 设置最大 token 限制（构建器模式）。
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Get endpoint.
    ///
    /// 获取 API Endpoint。
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Set endpoint.
    ///
    /// 设置 API Endpoint。
    pub fn set_endpoint(&mut self, endpoint: impl Into<StaticRefStr>) -> &mut Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set endpoint (builder pattern).
    ///
    /// 设置 API Endpoint（构建器模式）。
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
    metadata: Option<Metadata>,
    container: Option<Container>,
    stop_sequences: Option<Vec<StaticRefStr>>,
    temperature: Option<f32>,
    tool_choice: Option<ToolChoice>,
    top_k: Option<u32>,
    top_p: Option<f32>,
    thinking_param: Option<ThinkingConfigParam>,
    output_effort: Option<OutputEffort>,
    output_format: Option<OutputFormat>,
    output_config: Option<OutputConfig>,
    service_tier: Option<StaticRefStr>,
}

impl MessagesOptionalConfig {
    /// Create a new `MessagesOptionalConfig`.
    ///
    /// 创建新的 `MessagesOptionalConfig`。
    pub fn new() -> Self {
        Self::default()
    }

    /// Get metadata.
    ///
    /// 获取元数据。
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Set metadata.
    ///
    /// 设置元数据。
    pub fn set_metadata(&mut self, metadata: Option<Metadata>) -> &mut Self {
        self.metadata = metadata;
        self
    }

    /// Set metadata (builder pattern).
    ///
    /// 设置元数据（构建器模式）。
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get container.
    ///
    /// 获取容器信息。
    pub fn container(&self) -> Option<&Container> {
        self.container.as_ref()
    }

    /// Set container.
    ///
    /// 设置容器信息。
    pub fn set_container(&mut self, container: Option<Container>) -> &mut Self {
        self.container = container;
        self
    }

    /// Set container (builder pattern).
    ///
    /// 设置容器信息（构建器模式）。
    pub fn with_container(mut self, container: Container) -> Self {
        self.container = Some(container);
        self
    }

    /// Get stop sequences.
    ///
    /// 获取停止词列表。
    pub fn stop_sequences(&self) -> Option<&[StaticRefStr]> {
        self.stop_sequences.as_deref()
    }

    /// Set stop sequences.
    ///
    /// 设置停止词列表。
    pub fn set_stop_sequences(&mut self, stop_sequences: Option<Vec<StaticRefStr>>) -> &mut Self {
        self.stop_sequences = stop_sequences;
        self
    }

    /// Set stop sequences (builder pattern).
    ///
    /// 设置停止词列表（构建器模式）。
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<StaticRefStr>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Get temperature.
    ///
    /// 获取采样温度。
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Set temperature.
    ///
    /// 设置采样温度。
    pub fn set_temperature(&mut self, temperature: Option<f32>) -> &mut Self {
        self.temperature = temperature;
        self
    }

    /// Set temperature (builder pattern).
    ///
    /// 设置采样温度（构建器模式）。
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Get tool choice.
    ///
    /// 获取工具选择规则。
    pub fn tool_choice(&self) -> Option<&ToolChoice> {
        self.tool_choice.as_ref()
    }

    /// Set tool choice.
    ///
    /// 设置工具选择规则。
    pub fn set_tool_choice(&mut self, tool_choice: Option<ToolChoice>) -> &mut Self {
        self.tool_choice = tool_choice;
        self
    }

    /// Set tool choice (builder pattern).
    ///
    /// 设置工具选择规则（构建器模式）。
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Get top_k.
    ///
    /// 获取 Top-K 采样值。
    pub fn top_k(&self) -> Option<u32> {
        self.top_k
    }

    /// Set top_k.
    ///
    /// 设置 Top-K 采样值。
    pub fn set_top_k(&mut self, top_k: Option<u32>) -> &mut Self {
        self.top_k = top_k;
        self
    }

    /// Set top_k (builder pattern).
    ///
    /// 设置 Top-K 采样值（构建器模式）。
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Get top_p.
    ///
    /// 获取 Top-P 采样值。
    pub fn top_p(&self) -> Option<f32> {
        self.top_p
    }

    /// Set top_p.
    ///
    /// 设置 Top-P 采样值。
    pub fn set_top_p(&mut self, top_p: Option<f32>) -> &mut Self {
        self.top_p = top_p;
        self
    }

    /// Set top_p (builder pattern).
    ///
    /// 设置 Top-P 采样值（构建器模式）。
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Get thinking config parameter.
    ///
    /// 获取思考配置参数。
    pub fn thinking_param(&self) -> Option<&ThinkingConfigParam> {
        self.thinking_param.as_ref()
    }

    /// Set thinking config parameter.
    ///
    /// 设置思考配置参数。
    pub fn set_thinking_param(&mut self, thinking_param: Option<ThinkingConfigParam>) -> &mut Self {
        self.thinking_param = thinking_param;
        self
    }

    /// Set thinking config parameter (builder pattern).
    ///
    /// 设置思考配置参数（构建器模式）。
    pub fn with_thinking_param(mut self, thinking_param: ThinkingConfigParam) -> Self {
        self.thinking_param = Some(thinking_param);
        self
    }

    /// Get output reasoning effort.
    ///
    /// 获取输出思考强度。
    pub fn output_effort(&self) -> Option<&OutputEffort> {
        self.output_effort.as_ref()
    }

    /// Set output reasoning effort.
    ///
    /// 设置输出思考强度。
    pub fn set_output_effort(&mut self, effort: Option<OutputEffort>) -> &mut Self {
        self.output_effort = effort;
        self
    }

    /// Set output reasoning effort (builder pattern).
    ///
    /// 设置输出思考强度（构建器模式）。
    pub fn with_output_effort(mut self, effort: OutputEffort) -> Self {
        self.output_effort = Some(effort);
        self
    }

    /// Get output format.
    ///
    /// 获取输出格式配置。
    pub fn output_format(&self) -> Option<&OutputFormat> {
        self.output_format.as_ref()
    }

    /// Set output format.
    ///
    /// 设置输出格式配置。
    pub fn set_output_format(&mut self, format: Option<OutputFormat>) -> &mut Self {
        self.output_format = format;
        self
    }

    /// Set output format (builder pattern).
    ///
    /// 设置输出格式配置（构建器模式）。
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = Some(format);
        self
    }

    /// Get output config.
    ///
    /// 获取完整的输出配置对象。
    pub fn output_config(&self) -> Option<&OutputConfig> {
        self.output_config.as_ref()
    }

    /// Set output config.
    ///
    /// 设置完整的输出配置对象。
    pub fn set_output_config(&mut self, output_config: Option<OutputConfig>) -> &mut Self {
        self.output_config = output_config;
        self
    }

    /// Set output config (builder pattern).
    ///
    /// 设置完整的输出配置对象（构建器模式）。
    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = Some(output_config);
        self
    }

    /// Get service tier.
    ///
    /// 获取服务层级。
    pub fn service_tier(&self) -> Option<&str> {
        self.service_tier.as_deref()
    }

    /// Set service tier.
    ///
    /// 设置服务层级。
    pub fn set_service_tier(&mut self, service_tier: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.service_tier = service_tier.map(Into::into);
        self
    }

    /// Set service tier (builder pattern).
    ///
    /// 设置服务层级（构建器模式）。
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
    ///
    /// 创建新的 `MessagesConfig`。
    pub fn new(required: MessagesRequiredConfig) -> Self {
        Self {
            required,
            optional: MessagesOptionalConfig::default(),
        }
    }

    /// Get reference to required configuration.
    ///
    /// 获取必要配置的引用。
    pub fn required(&self) -> &MessagesRequiredConfig {
        &self.required
    }

    /// Get mutable reference to required configuration.
    ///
    /// 获取必要配置的可变引用。
    pub fn required_mut(&mut self) -> &mut MessagesRequiredConfig {
        &mut self.required
    }

    /// Get reference to optional configuration.
    ///
    /// 获取选填配置的引用。
    pub fn optional(&self) -> &MessagesOptionalConfig {
        &self.optional
    }

    /// Get mutable reference to optional configuration.
    ///
    /// 获取选填配置的可变引用。
    pub fn optional_mut(&mut self) -> &mut MessagesOptionalConfig {
        &mut self.optional
    }

    /// Set optional configuration (builder pattern).
    ///
    /// 设置选填配置（构建器模式）。
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
        let output_config =
            if self.optional.output_effort.is_some() || self.optional.output_format.is_some() {
                Some(OutputConfig {
                    effort: self.optional.output_effort,
                    format: self.optional.output_format,
                })
            } else {
                self.optional.output_config
            };

        MessagesRequest {
            model: self.required.model,
            messages,
            max_tokens: Some(self.required.max_tokens),
            system,
            metadata: self.optional.metadata,
            container: self.optional.container,
            stop_sequences: self.optional.stop_sequences,
            stream: Some(stream),
            temperature: self.optional.temperature,
            tool_choice,
            tools,
            top_k: self.optional.top_k,
            top_p: self.optional.top_p,
            thinking: self.optional.thinking_param,
            output_config,
            service_tier: self.optional.service_tier,
        }
    }
}

impl TryFrom<RequiredConfig> for MessagesRequiredConfig {
    type Error = AgentError;

    fn try_from(config: RequiredConfig) -> Result<Self, Self::Error> {
        let model = config
            .model_static()
            .ok_or_else(|| AgentError::Config("model is required".into()))?;
        let max_tokens = config
            .max_tokens()
            .ok_or_else(|| AgentError::Config("max_tokens is required".into()))?;
        let endpoint = config
            .endpoint_static()
            .ok_or_else(|| AgentError::Config("endpoint is required".into()))?;

        Ok(Self::new(model, max_tokens, endpoint))
    }
}

impl crate::agent::builder::AgentConfigTrait for MessagesConfig {
    type Required = MessagesRequiredConfig;
    type Optional = MessagesOptionalConfig;

    fn from_required(required: Self::Required) -> Self {
        Self::new(required)
    }

    fn with_optional(self, optional: Self::Optional) -> Self {
        self.with_optional(optional)
    }
}

impl TryFrom<OptionalConfig> for MessagesOptionalConfig {
    type Error = AgentError;

    fn try_from(config: OptionalConfig) -> Result<Self, Self::Error> {
        let OptionalConfig {
            temperature,
            top_p,
            top_k,
            frequency_penalty: _,
            presence_penalty: _,
            stop_sequences,
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
        if let Some(top_k) = top_k {
            optional.set_top_k(Some(top_k));
        }
        if let Some(stop) = stop_sequences {
            optional.set_stop_sequences(Some(stop));
        }
        if let Some(effort) = reasoning_effort {
            let output_effort = match effort {
                ReasoningEffort::None => OutputEffort::Low,
                ReasoningEffort::Minimal => OutputEffort::Low,
                ReasoningEffort::Low => OutputEffort::Low,
                ReasoningEffort::Medium => OutputEffort::Medium,
                ReasoningEffort::High => OutputEffort::High,
                ReasoningEffort::Xhigh => OutputEffort::Xhigh,
                ReasoningEffort::Max => OutputEffort::Max,
            };
            optional.set_output_effort(Some(output_effort));
        }
        Ok(optional)
    }
}

impl TryFrom<Config> for MessagesConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let (required, optional) = (config.required().clone(), config.optional().clone());
        let required = MessagesRequiredConfig::try_from(required)?;
        let optional = MessagesOptionalConfig::try_from(optional)?;
        Ok(Self::new(required).with_optional(optional))
    }
}
