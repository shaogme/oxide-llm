use oxide_llm_core::message::DeltaMessage;
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Level or effort of model reasoning/thinking.
///
/// 模型思考/推理的努力程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Disable reasoning effort.
    ///
    /// 禁用推理努力。
    None,
    /// Minimal reasoning effort.
    ///
    /// 极小程度的推理努力。
    Minimal,
    /// Low reasoning effort.
    ///
    /// 较低程度的推理努力。
    Low,
    /// Medium reasoning effort.
    ///
    /// 中等程度的推理努力。
    Medium,
    /// High reasoning effort.
    ///
    /// 较高程度的推理努力。
    High,
    /// Extra high reasoning effort.
    ///
    /// 超高程度的推理努力。
    Xhigh,
    /// Maximum reasoning effort.
    ///
    /// 最高程度的推理努力。
    Max,
}

/// Thinking configuration.
///
/// 思考配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThinkingConfig {
    /// 纯布尔控制
    Bool(bool),
    /// Token 预算数值控制
    Budget(u64),
    /// 结构化控制对象
    Full {
        /// 是否开启思考
        #[serde(default)]
        enabled: Option<bool>,
        /// 思考 Token 预算
        #[serde(default)]
        budget_tokens: Option<u64>,
    },
}

/// Unified generic agent configuration.
///
/// 统一通用代理配置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Model name.
    ///
    /// 模型名称。
    pub(crate) model: StaticRefStr,
    /// Maximum tokens to generate (Required for Anthropic protocol).
    ///
    /// 最大 token 限制（Anthropic 协议必须）。
    pub(crate) max_tokens: Option<u32>,
    pub(crate) endpoint: Option<StaticRefStr>,
    pub(crate) temperature: Option<f32>,
    pub(crate) top_p: Option<f32>,
    pub(crate) top_k: Option<u32>,
    pub(crate) frequency_penalty: Option<f32>,
    pub(crate) presence_penalty: Option<f32>,
    pub(crate) stop_sequences: Option<Vec<StaticRefStr>>,
    pub(crate) seed: Option<i64>,
    #[serde(alias = "effort")]
    pub(crate) reasoning_effort: Option<ReasoningEffort>,
    pub(crate) thinking: Option<ThinkingConfig>,
}

impl Config {
    /// Create a new `Config`.
    ///
    /// 创建新的 `Config`。
    pub fn new(model: impl Into<StaticRefStr>) -> Self {
        Self {
            model: model.into(),
            max_tokens: None,
            endpoint: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
            seed: None,
            reasoning_effort: None,
            thinking: None,
        }
    }

    /// Get model name.
    ///
    /// 获取模型名称。
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get model name as `StaticRefStr`.
    ///
    /// 获取 `StaticRefStr` 类型的模型名称。
    pub fn model_static(&self) -> StaticRefStr {
        self.model.clone()
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
    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    /// Set max tokens.
    ///
    /// 设置最大 token 限制。
    pub fn set_max_tokens(&mut self, max_tokens: Option<u32>) -> &mut Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set max tokens (builder pattern).
    ///
    /// 设置最大 token 限制（构建器模式）。
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
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

    /// Get frequency penalty.
    ///
    /// 获取频率惩罚值。
    pub fn frequency_penalty(&self) -> Option<f32> {
        self.frequency_penalty
    }

    /// Set frequency penalty.
    ///
    /// 设置频率惩罚值。
    pub fn set_frequency_penalty(&mut self, frequency_penalty: Option<f32>) -> &mut Self {
        self.frequency_penalty = frequency_penalty;
        self
    }

    /// Set frequency penalty (builder pattern).
    ///
    /// 设置频率惩罚值（构建器模式）。
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Get presence penalty.
    ///
    /// 获取存在惩罚值。
    pub fn presence_penalty(&self) -> Option<f32> {
        self.presence_penalty
    }

    /// Set presence penalty.
    ///
    /// 设置存在惩罚值。
    pub fn set_presence_penalty(&mut self, presence_penalty: Option<f32>) -> &mut Self {
        self.presence_penalty = presence_penalty;
        self
    }

    /// Set presence penalty (builder pattern).
    ///
    /// 设置存在惩罚值（构建器模式）。
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
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

    /// Get seed.
    ///
    /// 获取随机种子。
    pub fn seed(&self) -> Option<i64> {
        self.seed
    }

    /// Set seed.
    ///
    /// 设置随机种子。
    pub fn set_seed(&mut self, seed: Option<i64>) -> &mut Self {
        self.seed = seed;
        self
    }

    /// Set seed (builder pattern).
    ///
    /// 设置随机种子（构建器模式）。
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Get reasoning effort.
    ///
    /// 获取思考/推理努力程度。
    pub fn reasoning_effort(&self) -> Option<ReasoningEffort> {
        self.reasoning_effort
    }

    /// Set reasoning effort.
    ///
    /// 设置思考/推理努力程度。
    pub fn set_reasoning_effort(&mut self, reasoning_effort: Option<ReasoningEffort>) -> &mut Self {
        self.reasoning_effort = reasoning_effort;
        self
    }

    /// Set reasoning effort (builder pattern).
    ///
    /// 设置思考/推理努力程度（构建器模式）。
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
        self
    }

    /// Get thinking config.
    ///
    /// 获取思考配置。
    pub fn thinking(&self) -> Option<&ThinkingConfig> {
        self.thinking.as_ref()
    }

    /// Set thinking config.
    ///
    /// 设置思考配置。
    pub fn set_thinking(&mut self, thinking: Option<ThinkingConfig>) -> &mut Self {
        self.thinking = thinking;
        self
    }

    /// Set thinking config (builder pattern).
    ///
    /// 设置思考配置（构建器模式）。
    pub fn with_thinking(mut self, thinking: ThinkingConfig) -> Self {
        self.thinking = Some(thinking);
        self
    }
}

/// Alias for a raw delta callback hook.
///
/// 原始增量回调 Hook 类型别名。
pub type RawDeltaHook<RawDelta> = Box<dyn FnMut(&RawDelta) + Send + 'static>;

/// Alias for a parsed delta message callback hook.
///
/// 解析增量消息回调 Hook 类型别名。
pub type DeltaHook = Box<dyn FnMut(&DeltaMessage) + Send + 'static>;

/// Configuration for raw chat stream, including hooks.
///
/// 原始聊天流配置（包含 Hook）。
pub struct ChatStreamRawConfig<RawDelta> {
    on_raw_delta: Option<RawDeltaHook<RawDelta>>,
}

impl<RawDelta> Default for ChatStreamRawConfig<RawDelta> {
    fn default() -> Self {
        Self { on_raw_delta: None }
    }
}

impl<RawDelta> ChatStreamRawConfig<RawDelta> {
    /// Create a new empty `ChatStreamRawConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set hook called when a `RawDelta` arrives.
    ///
    /// 设置 `RawDelta` 到达时的 Hook。
    pub fn on_raw_delta<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&RawDelta) + Send + 'static,
    {
        self.on_raw_delta = Some(Box::new(hook));
        self
    }

    /// Take the `on_raw_delta` hook.
    pub fn take_on_raw_delta(&mut self) -> Option<RawDeltaHook<RawDelta>> {
        self.on_raw_delta.take()
    }
}

/// Configuration for parsed chat stream, including hooks.
///
/// 解析后的聊天流配置（包含 Hook）。
pub struct ChatStreamConfig<RawDelta> {
    on_raw_delta: Option<RawDeltaHook<RawDelta>>,
    on_delta: Option<DeltaHook>,
}

impl<RawDelta> Default for ChatStreamConfig<RawDelta> {
    fn default() -> Self {
        Self {
            on_raw_delta: None,
            on_delta: None,
        }
    }
}

impl<RawDelta> ChatStreamConfig<RawDelta> {
    /// Create a new empty `ChatStreamConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set hook called when a `RawDelta` arrives.
    ///
    /// 设置 `RawDelta` 到达时的 Hook。
    pub fn on_raw_delta<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&RawDelta) + Send + 'static,
    {
        self.on_raw_delta = Some(Box::new(hook));
        self
    }

    /// Set hook called after `RawDelta` is parsed into `DeltaMessage`.
    ///
    /// 设置 `RawDelta` 解析为 `DeltaMessage` 后的 Hook。
    pub fn on_delta<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&DeltaMessage) + Send + 'static,
    {
        self.on_delta = Some(Box::new(hook));
        self
    }

    /// Take the `on_raw_delta` hook.
    pub fn take_on_raw_delta(&mut self) -> Option<RawDeltaHook<RawDelta>> {
        self.on_raw_delta.take()
    }

    /// Take the `on_delta` hook.
    pub fn take_on_delta(&mut self) -> Option<DeltaHook> {
        self.on_delta.take()
    }
}
