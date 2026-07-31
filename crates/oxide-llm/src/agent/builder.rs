use crate::config::Config;
use crate::error::{AgentError, Result};
use oxide_llm_core::transport::Transport;
use std::marker::PhantomData;

/// Trait implemented by agent configs that can be created from generic `Config`.
///
/// 支持从标准 `Config` 转换的代理配置接口。
pub trait AgentConfigTrait: Sized + TryFrom<Config, Error = AgentError> {}

impl<T: Sized + TryFrom<Config, Error = AgentError>> AgentConfigTrait for T {}

/// Configuration state for `AgentBuilder`.
///
/// `AgentBuilder` 的配置状态。
#[derive(Debug, Default)]
enum AgentConfigState<C> {
    #[default]
    Unset,
    Config(C),
}

impl<C: Clone> Clone for AgentConfigState<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Unset => Self::Unset,
            Self::Config(cfg) => Self::Config(cfg.clone()),
        }
    }
}

/// Generic Agent Builder for building LLM agents with standard configuration flow.
///
/// 用于按标准配置流程构建 LLM 代理的通用 Agent 构建器。
pub struct AgentBuilder<T: Clone, C: AgentConfigTrait, A> {
    transport: T,
    state: AgentConfigState<C>,
    _marker: PhantomData<A>,
}

impl<T: Clone, C: AgentConfigTrait, A> Clone for AgentBuilder<T, C, A>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            state: self.state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Transport, C: AgentConfigTrait, A> AgentBuilder<T, C, A> {
    /// Create a new `AgentBuilder`.
    ///
    /// 创建一个新的 `AgentBuilder`。
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            state: AgentConfigState::Unset,
            _marker: PhantomData,
        }
    }

    /// Set configuration for the agent builder.
    ///
    /// 为代理构建器设置配置。
    pub fn with_raw_config(mut self, config: C) -> Self {
        self.state = AgentConfigState::Config(config);
        self
    }

    /// Set configuration for the agent builder using generic `Config`.
    ///
    /// 使用通用 `Config` 为代理构建器设置配置。
    pub fn with_config(mut self, config: Config) -> Result<Self> {
        self.state = AgentConfigState::Config(C::try_from(config)?);
        Ok(self)
    }

    /// Resolve transport and final agent configuration.
    ///
    /// 解析网络传输层和最终代理配置。
    pub fn build_config(self) -> Result<(T, C)> {
        match self.state {
            AgentConfigState::Config(config) => Ok((self.transport, config)),
            AgentConfigState::Unset => Err(AgentError::Config("configuration missing".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyConfig {
        model: String,
        temperature: Option<u32>,
    }

    impl TryFrom<Config> for DummyConfig {
        type Error = AgentError;
        fn try_from(config: Config) -> Result<Self> {
            let Config {
                model,
                max_tokens: _,
                endpoint: _,
                temperature: _,
                top_p: _,
                top_k: _,
                frequency_penalty: _,
                presence_penalty: _,
                stop_sequences: _,
                seed: _,
                reasoning_effort: _,
                thinking: _,
            } = config;

            Ok(Self {
                model: model.to_string(),
                temperature: None,
            })
        }
    }

    #[derive(Clone)]
    struct DummyTransport;

    impl oxide_llm_core::transport::Transport for DummyTransport {
        type Stream = futures::stream::Empty<
            std::result::Result<bytes::Bytes, oxide_llm_core::transport::TransportError>,
        >;
        type StreamFuture = std::future::Ready<
            std::result::Result<Self::Stream, oxide_llm_core::transport::TransportError>,
        >;

        fn send<Req, Res>(
            &self,
            _: oxide_llm_core::transport::TransportRequest<Req>,
        ) -> impl futures::Future<
            Output = std::result::Result<Res, oxide_llm_core::transport::TransportError>,
        > + Send
        where
            Req: serde::Serialize + Send + Sync,
            Res: serde::de::DeserializeOwned + Send + Sync,
        {
            async { todo!() }
        }

        fn stream<Req>(
            &self,
            _: oxide_llm_core::transport::TransportRequest<Req>,
        ) -> Self::StreamFuture
        where
            Req: serde::Serialize + Send + Sync,
        {
            std::future::ready(Ok(futures::stream::empty()))
        }
    }

    struct DummyAgent;

    #[test]
    fn test_builder_unset_returns_error() {
        let builder = AgentBuilder::<DummyTransport, DummyConfig, DummyAgent>::new(DummyTransport);
        assert!(builder.build_config().is_err());
    }

    #[test]
    fn test_builder_with_raw_config() {
        let builder = AgentBuilder::<DummyTransport, DummyConfig, DummyAgent>::new(DummyTransport);
        let config = DummyConfig {
            model: "custom".into(),
            temperature: Some(10),
        };
        let (_, resolved) = builder
            .with_raw_config(config.clone())
            .build_config()
            .unwrap();
        assert_eq!(resolved, config);
    }

    #[test]
    fn test_builder_with_generic_config() {
        let builder = AgentBuilder::<DummyTransport, DummyConfig, DummyAgent>::new(DummyTransport);
        let gen_config = Config::new("req-model");
        let (_, resolved) = builder
            .with_config(gen_config)
            .unwrap()
            .build_config()
            .unwrap();
        assert_eq!(
            resolved,
            DummyConfig {
                model: "req-model".into(),
                temperature: None,
            }
        );
    }
}
