pub mod agent {
    pub mod builder;
    pub mod claude;
    pub mod gemini;
    pub mod openai;
}

pub mod config;
pub mod error;
pub mod runner;
pub mod stream;
pub mod traits;

pub use config::{
    ChatStreamConfig, ChatStreamRawConfig, Config, OptionalConfig, ReasoningEffort, RequiredConfig,
};
pub use runner::{DefaultExecutor, Executor, Runner, SequentialExecutor};
pub use traits::{ChatAgent, DynChatAgent};

pub mod macros {
    pub use oxide_llm_macros::*;
}

#[cfg(feature = "transport")]
pub mod transport {
    #[cfg(feature = "transport-reqwest")]
    pub use oxide_llm_transport::reqwest;
}

pub mod core {
    pub use oxide_llm_core::*;
}

pub mod proto {
    pub use oxide_llm_core::reexports::oxide_llm_proto::*;
}

pub mod reexports {
    pub use ref_str;
    pub use serde_json;
}
