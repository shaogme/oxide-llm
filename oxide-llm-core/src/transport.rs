use error_set::error_set;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

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

/// HTTP Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Generic Transport Request
#[derive(Debug, Clone)]
pub struct TransportRequest<B> {
    pub method: Method,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub body: B,
}

impl<B> TransportRequest<B> {
    pub fn new(method: Method, endpoint: impl Into<String>, body: B) -> Self {
        Self {
            method,
            endpoint: endpoint.into(),
            headers: HashMap::new(),
            body,
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
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
#[trait_morph::morph(Send)]
pub trait Transport: Send + Sync + Clone {
    type Stream: futures::stream::Stream<Item = Result<bytes::Bytes, TransportError>>
        + Send
        + 'static;

    /// Sends a request and retrieval a response.
    ///
    /// # Parameters
    /// - `req`: The transport request containing method, endpoint, headers and body.
    ///
    /// # Returns
    /// - Returns the deserialized response structure `Res` on success.
    /// - Returns `TransportError` on failure.
    ///
    /// 发送请求并获取响应。
    ///
    /// # 参数
    /// - `req`: 包含方法、端点、头信息和请求体的传输层请求。
    ///
    /// # 返回
    /// - 成功时返回反序列化后的响应结构体 `Res`。
    /// - 失败时返回 `TransportError`。
    async fn send<Req, Res>(&self, req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync;

    /// Sends a request and returns a stream of bytes.
    ///
    /// # Parameters
    /// - `req`: The transport request containing method, endpoint, headers and body.
    ///
    /// # Returns
    /// - Returns a `Stream` producing `Bytes` on success.
    /// - Returns `TransportError` on failure.
    ///
    /// 发送请求并返回字节流。
    ///
    /// # 参数
    /// - `req`: 包含方法、端点、头信息和请求体的传输层请求。
    ///
    /// # 返回
    /// - 成功时返回产生 `Bytes` 的 `Stream`。
    /// - 失败时返回 `TransportError`。
    async fn stream<Req>(&self, req: TransportRequest<Req>) -> Result<Self::Stream, TransportError>
    where
        Req: Serialize + Send + Sync;
}

// Authorization Middleware
#[derive(Clone, Debug)]
pub struct AuthorizationLayer<T> {
    inner: T,
    api_key: String,
}

impl<T> AuthorizationLayer<T> {
    pub fn new(inner: T, api_key: impl Into<String>) -> Self {
        Self {
            inner,
            api_key: api_key.into(),
        }
    }
}

impl<T: Transport> Transport for AuthorizationLayer<T> {
    type Stream = T::Stream;

    async fn send<Req, Res>(&self, mut req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        req.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        self.inner.send(req).await
    }

    async fn stream<Req>(
        &self,
        mut req: TransportRequest<Req>,
    ) -> Result<Self::Stream, TransportError>
    where
        Req: Serialize + Send + Sync,
    {
        req.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        self.inner.stream(req).await
    }
}

// BaseUrl Middleware
#[derive(Clone, Debug)]
pub struct BaseUrlLayer<T> {
    inner: T,
    base_url: String,
}

impl<T> BaseUrlLayer<T> {
    pub fn new(inner: T, base_url: impl Into<String>) -> Self {
        let mut base_url = base_url.into();
        if base_url.ends_with('/') {
            base_url.pop();
        }
        Self { inner, base_url }
    }
}

impl<T: Transport> Transport for BaseUrlLayer<T> {
    type Stream = T::Stream;

    async fn send<Req, Res>(&self, mut req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        if !req.endpoint.starts_with("http") {
            let endpoint = req.endpoint.trim_start_matches('/');
            req.endpoint = format!("{}/{}", self.base_url, endpoint);
        }
        self.inner.send(req).await
    }

    async fn stream<Req>(
        &self,
        mut req: TransportRequest<Req>,
    ) -> Result<Self::Stream, TransportError>
    where
        Req: Serialize + Send + Sync,
    {
        if !req.endpoint.starts_with("http") {
            let endpoint = req.endpoint.trim_start_matches('/');
            req.endpoint = format!("{}/{}", self.base_url, endpoint);
        }
        self.inner.stream(req).await
    }
}
