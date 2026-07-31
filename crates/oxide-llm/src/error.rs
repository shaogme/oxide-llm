use diagweave::set;
use oxide_llm_core::mapper::MapperError;
use oxide_llm_core::transport::TransportError;

set! {
    pub AgentError = {
        #[display("Transport error: {0}")]
        Transport(TransportError),
        #[display("Mapper conversion error: {0}")]
        Mapper(MapperError),
        #[display("JSON error: {0}")]
        Json(serde_json::Error),
        #[display("UTF-8 error: {0}")]
        Utf8(std::str::Utf8Error),
        #[display("IO error: {0}")]
        Io(std::io::Error),
        #[display("Trace error: {0}")]
        Trace(String),
        #[display("Stream already polled: {0}")]
        AlreadyPolled(String),
        #[display("Tool execution error: {0}")]
        ToolExecution(String),
        #[display("Configuration error: {0}")]
        Config(String),
        #[display("Invalid stream data error: {0}")]
        StreamData(String),
    }
}

pub type Result<T> = std::result::Result<T, AgentError>;
