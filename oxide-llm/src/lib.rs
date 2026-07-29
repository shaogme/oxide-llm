pub mod agent {
    pub mod claude;
    pub mod gemini;
    pub mod openai;
}

pub mod config;
pub mod error;
pub mod runner;
pub mod stream;
pub mod traits;

pub use config::{ChatStreamConfig, ChatStreamRawConfig};
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

pub mod reexports {
    pub use serde_json;
}
