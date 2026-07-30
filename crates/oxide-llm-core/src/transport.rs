use diagweave::set;
use serde::{Serialize, de::DeserializeOwned};
use std::{borrow::Cow, collections::HashMap};

set! {
    pub TransportError = {
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
    pub endpoint: Cow<'static, str>,
    pub headers: HashMap<Cow<'static, str>, Cow<'static, str>>,
    pub body: B,
}

impl<B> TransportRequest<B> {
    pub fn new(method: Method, endpoint: impl Into<Cow<'static, str>>, body: B) -> Self {
        Self {
            method,
            endpoint: endpoint.into(),
            headers: HashMap::new(),
            body,
        }
    }

    pub fn header(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
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
pub trait Transport: Send + Sync + Clone + 'static {
    type Stream: futures::stream::Stream<Item = Result<bytes::Bytes, TransportError>>
        + Send
        + 'static
        + Unpin;
    type StreamFuture: std::future::Future<Output = Result<Self::Stream, TransportError>> + Send;

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
    fn stream<Req>(&self, req: TransportRequest<Req>) -> Self::StreamFuture
    where
        Req: Serialize + Send + Sync + 'static;
}

// Authorization Middleware
#[derive(Clone, Debug)]
pub struct AuthorizationLayer<T> {
    inner: T,
    api_key: Cow<'static, str>,
}

impl<T> AuthorizationLayer<T> {
    pub fn new(inner: T, api_key: impl Into<Cow<'static, str>>) -> Self {
        Self {
            inner,
            api_key: api_key.into(),
        }
    }
}

impl<T: Transport> Transport for AuthorizationLayer<T> {
    type Stream = T::Stream;
    type StreamFuture = T::StreamFuture;

    async fn send<Req, Res>(&self, mut req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        req.headers.insert(
            "Authorization".into(),
            format!("Bearer {}", self.api_key).into(),
        );
        self.inner.send(req).await
    }

    fn stream<Req>(&self, mut req: TransportRequest<Req>) -> Self::StreamFuture
    where
        Req: Serialize + Send + Sync + 'static,
    {
        req.headers.insert(
            "Authorization".into(),
            format!("Bearer {}", self.api_key).into(),
        );
        self.inner.stream(req)
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
    type StreamFuture = T::StreamFuture;

    async fn send<Req, Res>(&self, mut req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        if !req.endpoint.starts_with("http") {
            let endpoint = req.endpoint.trim_start_matches('/');
            req.endpoint = format!("{}/{}", self.base_url, endpoint).into();
        }
        self.inner.send(req).await
    }

    fn stream<Req>(&self, mut req: TransportRequest<Req>) -> Self::StreamFuture
    where
        Req: Serialize + Send + Sync + 'static,
    {
        if !req.endpoint.starts_with("http") {
            let endpoint = req.endpoint.trim_start_matches('/');
            req.endpoint = format!("{}/{}", self.base_url, endpoint).into();
        }
        self.inner.stream(req)
    }
}

/// Extension trait for `Transport` to provide a fluent interface for layering.
///
/// 为 `Transport` 提供流式接口的扩展 Trait。
pub trait TransportExt: Transport + Sized {
    /// Adds an authorization layer to the transport.
    ///
    /// 为传输层添加认证层。
    fn with_authorization(self, api_key: impl Into<Cow<'static, str>>) -> AuthorizationLayer<Self> {
        AuthorizationLayer::new(self, api_key)
    }

    /// Adds a base URL layer to the transport.
    ///
    /// 为传输层添加 Base URL 层。
    fn with_base_url(self, base_url: impl Into<String>) -> BaseUrlLayer<Self> {
        BaseUrlLayer::new(self, base_url)
    }
}

impl<T: Transport> TransportExt for T {}
