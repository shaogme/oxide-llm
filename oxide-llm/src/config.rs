use oxide_llm_core::message::DeltaMessage;

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
