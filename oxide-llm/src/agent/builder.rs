use crate::config::{Config, OptionalConfig, RequiredConfig};
use crate::error::{AgentError, Result};
use oxide_llm_core::transport::Transport;
use std::marker::PhantomData;

/// Trait implemented by agent configs that support standard required and optional builder configuration.
///
/// 支持标准必要与可选配置构建的代理配置接口。
pub trait AgentConfigTrait: Sized + TryFrom<Config, Error = AgentError> {
    /// The required configuration type.
    type Required: TryFrom<RequiredConfig, Error = AgentError>;
    /// The optional configuration type.
    type Optional: TryFrom<OptionalConfig, Error = AgentError>;

    /// Create config from required configuration.
    fn from_required(required: Self::Required) -> Self;

    /// Apply optional configuration to the config.
    fn with_optional(self, optional: Self::Optional) -> Self;
}

/// Configuration state for `AgentBuilder`.
///
/// `AgentBuilder` 的配置状态。
#[derive(Debug, Default)]
enum AgentConfigState<C: AgentConfigTrait> {
    #[default]
    Unset,
    Full(C),
    Components {
        required: Option<C::Required>,
        optional: Option<C::Optional>,
    },
}

impl<C: AgentConfigTrait> Clone for AgentConfigState<C>
where
    C: Clone,
    C::Required: Clone,
    C::Optional: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Unset => Self::Unset,
            Self::Full(cfg) => Self::Full(cfg.clone()),
            Self::Components { required, optional } => Self::Components {
                required: required.clone(),
                optional: optional.clone(),
            },
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
    C::Required: Clone,
    C::Optional: Clone,
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

    /// Set raw configuration for the agent builder.
    ///
    /// 为代理构建器设置原始配置。
    pub fn with_raw_config(mut self, config: C) -> Self {
        self.state = AgentConfigState::Full(config);
        self
    }

    /// Set configuration for the agent builder using generic `Config`.
    ///
    /// 使用通用 `Config` 为代理构建器设置配置。
    pub fn with_config(mut self, config: Config) -> Result<Self> {
        self.state = AgentConfigState::Full(C::try_from(config)?);
        Ok(self)
    }

    /// Set required configuration for the agent builder.
    ///
    /// 为代理构建器设置必需配置。
    pub fn with_required_config(mut self, required: C::Required) -> Self {
        self.state = match self.state {
            AgentConfigState::Components { optional, .. } => AgentConfigState::Components {
                required: Some(required),
                optional,
            },
            _ => AgentConfigState::Components {
                required: Some(required),
                optional: None,
            },
        };
        self
    }

    /// Set required configuration for the agent builder using generic `RequiredConfig`.
    ///
    /// 使用通用 `RequiredConfig` 为代理构建器设置必需配置。
    pub fn with_raw_required_config(self, required: RequiredConfig) -> Result<Self> {
        let required = C::Required::try_from(required)?;
        Ok(self.with_required_config(required))
    }

    /// Set optional configuration for the agent builder.
    ///
    /// 为代理构建器设置可选配置。
    pub fn with_optional_config(mut self, optional: C::Optional) -> Self {
        self.state = match self.state {
            AgentConfigState::Full(config) => {
                AgentConfigState::Full(config.with_optional(optional))
            }
            AgentConfigState::Components { required, .. } => AgentConfigState::Components {
                required,
                optional: Some(optional),
            },
            AgentConfigState::Unset => AgentConfigState::Components {
                required: None,
                optional: Some(optional),
            },
        };
        self
    }

    /// Set optional configuration for the agent builder using generic `OptionalConfig`.
    ///
    /// 使用通用 `OptionalConfig` 为代理构建器设置可选配置。
    pub fn with_raw_optional_config(self, optional: OptionalConfig) -> Result<Self> {
        let optional = C::Optional::try_from(optional)?;
        Ok(self.with_optional_config(optional))
    }

    /// Resolve transport and final agent configuration.
    ///
    /// 解析网络传输层和最终代理配置。
    pub fn build_config(self) -> Result<(T, C)> {
        let config = match self.state {
            AgentConfigState::Full(config) => config,
            AgentConfigState::Components {
                required: Some(required),
                optional,
            } => {
                let mut cfg = C::from_required(required);
                if let Some(optional) = optional {
                    cfg = cfg.with_optional(optional);
                }
                cfg
            }
            _ => return Err(AgentError::Config("required configuration missing".into())),
        };

        Ok((self.transport, config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyRequired {
        model: String,
    }

    impl TryFrom<RequiredConfig> for DummyRequired {
        type Error = AgentError;
        fn try_from(_: RequiredConfig) -> Result<Self> {
            Ok(Self {
                model: "test-model".into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyOptional {
        temperature: Option<u32>,
    }

    impl TryFrom<OptionalConfig> for DummyOptional {
        type Error = AgentError;
        fn try_from(_: OptionalConfig) -> Result<Self> {
            Ok(Self {
                temperature: Some(42),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyConfig {
        model: String,
        temperature: Option<u32>,
    }

    impl TryFrom<Config> for DummyConfig {
        type Error = AgentError;
        fn try_from(_: Config) -> Result<Self> {
            Ok(Self {
                model: "config-model".into(),
                temperature: None,
            })
        }
    }

    impl AgentConfigTrait for DummyConfig {
        type Required = DummyRequired;
        type Optional = DummyOptional;

        fn from_required(required: Self::Required) -> Self {
            Self {
                model: required.model,
                temperature: None,
            }
        }

        fn with_optional(mut self, optional: Self::Optional) -> Self {
            self.temperature = optional.temperature;
            self
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
    fn test_builder_with_components() {
        let builder = AgentBuilder::<DummyTransport, DummyConfig, DummyAgent>::new(DummyTransport);
        let req = DummyRequired {
            model: "req-model".into(),
        };
        let opt = DummyOptional {
            temperature: Some(99),
        };
        let (_, resolved) = builder
            .with_required_config(req)
            .with_optional_config(opt)
            .build_config()
            .unwrap();
        assert_eq!(
            resolved,
            DummyConfig {
                model: "req-model".into(),
                temperature: Some(99),
            }
        );
    }
}
