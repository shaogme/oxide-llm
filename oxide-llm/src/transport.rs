use error_set::error_set;
use serde::{Serialize, de::DeserializeOwned};

error_set! {
    TransportError := {
        #[display("Codec error: {message}")]
        Codec { message: String },
        #[display("Network error: {message}")]
        Network { message: String },
        #[display("API error {status}: {message}")]
        Api {
            status: u16,
            message: String
        },
        #[display("Authentication error: {message}")]
        Auth { message: String },
        #[display("Other error: {message}")]
        Other { message: String },
    }
}

/// Abstract interface for the transport layer.
///
/// This trait decouples the upper-layer business logic (Agent) from the underlying communication protocols (HTTPClient, WebSocket, etc.).
/// Implementers are responsible for handling Base URL, API Key injection, header management, and specific network I/O.
///
/// 传输层抽象接口。
///
/// 该 Trait 将上层业务逻辑（Agent）与底层的通信协议（HTTPClient, WebSocket 等）解耦。
/// 实现者负责处理 Base URL、API Key 注入、请求头管理以及具体的网络 I/O。
#[trait_variant::make(Transport: Send)]
pub trait LocalTransport: Send + Sync + Clone {
    /// Sends a request and retrieves a response.
    ///
    /// # Parameters
    /// - `endpoint`: The relative path or endpoint identifier of the API (e.g., "/v1/chat/completions").
    /// - `payload`: The request body, which must implement Serialize.
    ///
    /// # Returns
    /// - Returns the deserialized response structure `Res` on success.
    /// - Returns `TransportError` on failure.
    ///
    /// 发送请求并获取响应。
    ///
    /// # 参数
    /// - `endpoint`: API 的相对路径或端点标识（例如 "/v1/chat/completions"）。
    /// - `payload`: 请求体，需要实现 Serialize。
    ///
    /// # 返回
    /// - 成功时返回反序列化后的响应结构体 `Res`。
    /// - 失败时返回 `TransportError`。
    async fn send<Req, Res>(&self, endpoint: &str, payload: Req) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync;
}
