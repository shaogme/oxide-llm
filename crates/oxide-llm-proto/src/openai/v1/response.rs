/// Streaming chunk structures for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的流式 Chunk 结构体。
pub mod chunk;

/// Configuration and parameter structures for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的配置与参数结构体。
pub mod config;

/// Log probability statistics for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的对数概率统计信息。
pub mod log;

/// Request structures and payload models for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的请求结构体与载荷模型。
pub mod request;

/// Response structures and payload models for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的响应结构体与载荷模型。
#[allow(clippy::module_inception)]
pub mod response;

/// Tool definition and choice models for OpenAI v1 Response API.
///
/// OpenAI v1 Response API 的工具定义与选择模型。
pub mod tool;

pub use chunk::*;
pub use config::*;
pub use log::*;
pub use request::*;
pub use response::*;
pub use tool::*;
