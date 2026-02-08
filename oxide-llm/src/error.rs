use error_set::error_set;
use oxide_llm_core::mapper::MapperError;
use oxide_llm_core::transport::TransportError;

error_set! {
    AgentError := {
        #[display("Transport error: {0}")]
        Transport(TransportError),
        #[display("Mapper conversion error: {0}")]
        Mapper(MapperError),
        #[display("JSON error: {0}")]
        Json(serde_json::Error),
        #[display("UTF-8 error: {0}")]
        Utf8(std::str::Utf8Error),
    }
}

pub type Result<T> = std::result::Result<T, AgentError>;
