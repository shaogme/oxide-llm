use crate::{
    config::{Config, ReasoningEffort},
    error::AgentError,
};
use oxide_llm_proto::claude::v1::messages::{
    Container, Message as ClaudeMessage, MessagesRequest, Metadata, OutputConfig, OutputEffort,
    OutputFormat, SystemPrompt, ThinkingConfigParam, Tool, ToolChoice,
};
use ref_str::StaticRefStr;

/// Configuration for Claude Messages Agent.
///
/// Claude Messages 代理配置。
#[derive(Debug, Clone)]
pub struct MessagesConfig {
    model: StaticRefStr,
    max_tokens: u32,
    endpoint: Option<StaticRefStr>,
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

impl MessagesConfig {
    /// Create a new `MessagesConfig`.
    ///
    /// 创建新的 `MessagesConfig`。
    pub fn new(model: impl Into<StaticRefStr>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            max_tokens,
            endpoint: None,
            metadata: None,
            container: None,
            stop_sequences: None,
            temperature: None,
            tool_choice: None,
            top_k: None,
            top_p: None,
            thinking_param: None,
            output_effort: None,
            output_format: None,
            output_config: None,
            service_tier: None,
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
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// Get endpoint as `StaticRefStr`.
    ///
    /// 获取 `StaticRefStr` 类型的 API Endpoint。
    pub fn endpoint_static(&self) -> Option<StaticRefStr> {
        self.endpoint.clone()
    }

    /// Set endpoint.
    ///
    /// 设置 API Endpoint。
    pub fn set_endpoint(&mut self, endpoint: Option<impl Into<StaticRefStr>>) -> &mut Self {
        self.endpoint = endpoint.map(Into::into);
        self
    }

    /// Set endpoint (builder pattern).
    ///
    /// 设置 API Endpoint（构建器模式）。
    pub fn with_endpoint(mut self, endpoint: impl Into<StaticRefStr>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
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
        let output_config = if self.output_effort.is_some() || self.output_format.is_some() {
            Some(OutputConfig {
                effort: self.output_effort,
                format: self.output_format,
            })
        } else {
            self.output_config
        };

        MessagesRequest {
            model: self.model,
            messages,
            max_tokens: Some(self.max_tokens),
            system,
            metadata: self.metadata,
            container: self.container,
            stop_sequences: self.stop_sequences,
            stream: Some(stream),
            temperature: self.temperature,
            tool_choice,
            tools,
            top_k: self.top_k,
            top_p: self.top_p,
            thinking: self.thinking_param,
            output_config,
            service_tier: self.service_tier,
        }
    }
}

impl TryFrom<Config> for MessagesConfig {
    type Error = AgentError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let max_tokens = config.max_tokens().ok_or_else(|| {
            AgentError::Config("max_tokens is required for Anthropic protocol".into())
        })?;

        let mut msg_config = Self::new(config.model_static(), max_tokens);

        if let Some(ep) = config.endpoint_static() {
            msg_config.set_endpoint(Some(ep));
        }
        if let Some(temp) = config.temperature() {
            msg_config.set_temperature(Some(temp));
        }
        if let Some(top_p) = config.top_p() {
            msg_config.set_top_p(Some(top_p));
        }
        if let Some(top_k) = config.top_k() {
            msg_config.set_top_k(Some(top_k));
        }
        if let Some(stop) = config.stop_sequences() {
            msg_config.set_stop_sequences(Some(stop.to_vec()));
        }
        if let Some(effort) = config.reasoning_effort() {
            let output_effort = match effort {
                ReasoningEffort::None => OutputEffort::Low,
                ReasoningEffort::Minimal => OutputEffort::Low,
                ReasoningEffort::Low => OutputEffort::Low,
                ReasoningEffort::Medium => OutputEffort::Medium,
                ReasoningEffort::High => OutputEffort::High,
                ReasoningEffort::Xhigh => OutputEffort::Xhigh,
                ReasoningEffort::Max => OutputEffort::Max,
            };
            msg_config.set_output_effort(Some(output_effort));
        }

        Ok(msg_config)
    }
}
