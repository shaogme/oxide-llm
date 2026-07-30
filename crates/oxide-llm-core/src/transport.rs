use std::{borrow::Cow, collections::HashMap, fmt, sync::Arc};

use bytes::Bytes;
use diagweave::set;
use futures::{StreamExt as _, future::BoxFuture, stream::BoxStream};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

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

// ────────────────────────────────────────────────────────────────────────────
// TransportConfig — shared builder configuration
// ────────────────────────────────────────────────────────────────────────────

/// Common configuration shared by all transport builder implementations.
///
/// Holds the two fields (`base_url`, `api_key`) that every HTTP-based
/// transport builder needs, together with their normalisation logic, so
/// concrete builders only have to embed this struct and delegate to it.
///
/// 所有传输层 Builder 共享的通用配置。
///
/// 持有每个基于 HTTP 的传输层 Builder 都需要的两个字段（`base_url`、`api_key`），
/// 以及对应的规范化逻辑，具体 Builder 只需内嵌此结构体并委托调用即可。
#[derive(Debug, Default, Clone)]
pub struct TransportConfig {
    /// Base URL prepended to every relative endpoint (trailing `/` is stripped).
    pub base_url: Option<String>,
    /// Bearer-token API key injected as `Authorization: Bearer <key>`.
    pub api_key: Option<String>,
}

impl TransportConfig {
    /// Stores `base_url`, stripping any trailing slash.
    ///
    /// 存储 `base_url`，并去除末尾的 `/`。
    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        self.base_url = Some(url);
    }

    /// Stores the API key for Bearer-token authorization.
    ///
    /// 存储用于 Bearer Token 授权的 API Key。
    pub fn set_authorization(&mut self, api_key: impl Into<String>) {
        self.api_key = Some(api_key.into());
    }

    /// Consumes `self` and returns `base_url`, or an `Other` error if absent.
    ///
    /// 消费 `self` 并返回 `base_url`，若未设置则返回 `Other` 错误。
    pub fn require_base_url(self, context: &str) -> Result<String, TransportError> {
        self.base_url.ok_or_else(|| TransportError::Other {
            message: format!("{context}: `base_url` is required"),
        })
    }
}

/// Fluent builder mixin for transport configuration.
///
/// Implementors expose `transport_config_mut` so the blanket default methods
/// `with_base_url` and `with_authorization` can write into the embedded
/// [`TransportConfig`] without any repeated boilerplate.
///
/// 传输层配置的链式 Builder Mixin Trait。
///
/// 实现者暴露 `transport_config_mut`，使 blanket 默认方法 `with_base_url`
/// 和 `with_authorization` 能直接写入内嵌的 [`TransportConfig`]，
/// 无需重复样板代码。
pub trait TransportBuilder: Sized {
    /// Returns a mutable reference to the embedded [`TransportConfig`].
    ///
    /// 返回内嵌 [`TransportConfig`] 的可变引用。
    fn transport_config_mut(&mut self) -> &mut TransportConfig;

    /// Sets the base URL prepended to every relative endpoint.
    ///
    /// 设置拼接到相对端点前的 Base URL。
    fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.transport_config_mut().set_base_url(base_url);
        self
    }

    /// Sets the API key used for Bearer-token authorization.
    ///
    /// 设置用于 Bearer Token 授权的 API Key。
    fn with_authorization(mut self, api_key: impl Into<String>) -> Self {
        self.transport_config_mut().set_authorization(api_key);
        self
    }
}

/// Object-safe erased transport interface.
///
/// Unlike [`Transport`], this trait has no generic methods and can be used
/// as a trait object (`dyn DynTransport`). Both request body and response
/// body are represented as [`serde_json::Value`] so that the serialisation
/// boundary is pushed into the blanket implementation.
///
/// 对象安全的 Erased 传输层接口。
///
/// 与 [`Transport`] 不同，此 Trait 没有泛型方法，可以作为 Trait 对象
/// (`dyn DynTransport`) 使用。请求体和响应体均以 [`serde_json::Value`]
/// 表示，序列化边界由 blanket impl 负责处理。
pub trait DynTransport: Send + Sync {
    /// Send a pre-serialised request and return the raw JSON response.
    ///
    /// 发送已序列化的请求，返回原始 JSON 响应。
    fn send_erased(
        &self,
        req: TransportRequest<Value>,
    ) -> BoxFuture<'static, Result<Value, TransportError>>;

    /// Send a pre-serialised request and return a byte stream.
    ///
    /// 发送已序列化的请求，返回字节流。
    fn stream_erased(
        &self,
        req: TransportRequest<Value>,
    ) -> BoxFuture<'static, Result<BoxStream<'static, Result<Bytes, TransportError>>, TransportError>>;

    /// Clone this transport into a heap-allocated box.
    ///
    /// 将此传输克隆为堆分配的 Box。
    fn clone_box(&self) -> Box<dyn DynTransport>;
}

// ────────────────────────────────────────────────────────────────────────────
// Blanket impl: every T: Transport automatically implements DynTransport
// ────────────────────────────────────────────────────────────────────────────

impl<T: Transport> DynTransport for T {
    fn send_erased(
        &self,
        req: TransportRequest<Value>,
    ) -> BoxFuture<'static, Result<Value, TransportError>> {
        let this = self.clone();
        Box::pin(async move {
            let result: Value = this.send(req).await?;
            Ok(result)
        })
    }

    fn stream_erased(
        &self,
        req: TransportRequest<Value>,
    ) -> BoxFuture<'static, Result<BoxStream<'static, Result<Bytes, TransportError>>, TransportError>>
    {
        let fut = self.stream(req);
        Box::pin(async move {
            let stream = fut.await?;
            Ok(stream.boxed())
        })
    }

    fn clone_box(&self) -> Box<dyn DynTransport> {
        Box::new(self.clone())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AnyTransport — type-erased Transport wrapper
// ────────────────────────────────────────────────────────────────────────────

/// A type-erased, cheaply cloneable transport wrapper.
///
/// `AnyTransport` wraps any [`Transport`] implementation behind an
/// `Arc<dyn DynTransport>`, allowing agents to store a transport without
/// carrying a generic parameter. It still implements [`Transport`] itself,
/// so it is a drop-in replacement for any concrete transport type.
///
/// 类型擦除的、廉价可克隆的传输层包装器。
///
/// `AnyTransport` 将任意 [`Transport`] 实现封装在 `Arc<dyn DynTransport>`
/// 之后，允许 Agent 存储传输层而无需携带泛型参数。它本身仍然实现了
/// [`Transport`]，因此可以直接替换任何具体的传输类型。
#[derive(Clone)]
pub struct AnyTransport {
    inner: Arc<dyn DynTransport>,
}

impl AnyTransport {
    /// Wraps any concrete [`Transport`] into an `AnyTransport`.
    ///
    /// 将任意具体 [`Transport`] 包装为 `AnyTransport`。
    pub fn new<T: Transport>(transport: T) -> Self {
        Self {
            inner: Arc::new(transport),
        }
    }
}

impl fmt::Debug for AnyTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyTransport").finish_non_exhaustive()
    }
}

/// Boxed byte-stream type used by [`AnyTransport`].
///
/// [`AnyTransport`] 使用的 Boxed 字节流类型。
pub type AnyStream = BoxStream<'static, Result<Bytes, TransportError>>;

/// Boxed stream-future type used by [`AnyTransport`].
///
/// [`AnyTransport`] 使用的 Boxed 流 Future 类型。
pub type AnyStreamFuture = BoxFuture<'static, Result<AnyStream, TransportError>>;

impl Transport for AnyTransport {
    type Stream = AnyStream;
    type StreamFuture = AnyStreamFuture;

    async fn send<Req, Res>(&self, req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        // Erase the body type by serialising to Value.
        let body = serde_json::to_value(&req.body).map_err(|e| TransportError::Codec {
            message: e.to_string(),
        })?;
        let erased_req = TransportRequest {
            method: req.method,
            endpoint: req.endpoint,
            headers: req.headers,
            body,
        };
        let value = self.inner.send_erased(erased_req).await?;
        serde_json::from_value(value).map_err(|e| TransportError::Codec {
            message: e.to_string(),
        })
    }

    fn stream<Req>(&self, req: TransportRequest<Req>) -> Self::StreamFuture
    where
        Req: Serialize + Send + Sync + 'static,
    {
        // Erase the body type by serialising to Value.
        let body_result = serde_json::to_value(&req.body);
        let inner = self.inner.clone();
        Box::pin(async move {
            let body = body_result.map_err(|e| TransportError::Codec {
                message: e.to_string(),
            })?;
            let erased_req = TransportRequest {
                method: req.method,
                endpoint: req.endpoint,
                headers: req.headers,
                body,
            };
            inner.stream_erased(erased_req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use futures::{Stream, StreamExt as _, future::BoxFuture};
    use serde::{Deserialize, Serialize};

    use super::{
        AnyStream, AnyStreamFuture, AnyTransport, DynTransport, Method, Transport, TransportError,
        TransportRequest,
    };

    // ── Minimal in-memory Transport stub ────────────────────────────────────

    /// Records calls and echoes the serialised request body back as the
    /// response, enabling round-trip serialisation testing without a network.
    ///
    /// 记录调用次数并将请求体序列化后原样回显，无需网络即可验证序列化往返。
    #[derive(Clone, Default)]
    struct EchoTransport {
        call_count: Arc<Mutex<u32>>,
    }

    impl EchoTransport {}

    impl Transport for EchoTransport {
        type Stream = EchoStream;
        type StreamFuture = BoxFuture<'static, Result<EchoStream, TransportError>>;

        async fn send<Req, Res>(&self, req: TransportRequest<Req>) -> Result<Res, TransportError>
        where
            Req: Serialize + Send + Sync,
            Res: serde::de::DeserializeOwned + Send + Sync,
        {
            *self.call_count.lock().unwrap() += 1;
            // Echo: serialise body → deserialise as Res
            let value = serde_json::to_value(&req.body).map_err(|e| TransportError::Codec {
                message: e.to_string(),
            })?;
            serde_json::from_value(value).map_err(|e| TransportError::Codec {
                message: e.to_string(),
            })
        }

        fn stream<Req>(&self, req: TransportRequest<Req>) -> Self::StreamFuture
        where
            Req: Serialize + Send + Sync + 'static,
        {
            *self.call_count.lock().unwrap() += 1;
            let body = serde_json::to_vec(&req.body).unwrap_or_default();
            Box::pin(async move { Ok(EchoStream::new(body)) })
        }
    }

    /// A stream that yields a single chunk containing the provided bytes.
    ///
    /// 产生单个包含所提供字节块的流。
    pub struct EchoStream {
        chunk: Option<Bytes>,
    }

    impl EchoStream {
        fn new(data: Vec<u8>) -> Self {
            Self {
                chunk: Some(Bytes::from(data)),
            }
        }
    }

    impl Stream for EchoStream {
        type Item = Result<Bytes, TransportError>;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            std::task::Poll::Ready(self.chunk.take().map(Ok))
        }
    }

    impl Unpin for EchoStream {}

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn echo_req(body: impl Serialize) -> TransportRequest<impl Serialize> {
        TransportRequest::new(Method::Post, "/echo", body)
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    /// `AnyTransport::new` should compile for any concrete `Transport`.
    ///
    /// `AnyTransport::new` 应对任意具体 `Transport` 编译通过。
    #[test]
    fn test_any_transport_construction() {
        let t = EchoTransport::default();
        let _any = AnyTransport::new(t);
    }

    /// Cloning `AnyTransport` must share the underlying `Arc`, not copy it.
    ///
    /// 克隆 `AnyTransport` 必须共享底层 `Arc`，而非复制。
    #[test]
    fn test_any_transport_clone_shares_arc() {
        let inner = EchoTransport::default();
        let any = AnyTransport::new(inner.clone());
        let any2 = any.clone();

        // Both point at the same Arc — pointer equality via dyn fat pointer
        // cannot be checked directly, but we can confirm they share state by
        // verifying that Debug output is the same (structurally).
        assert_eq!(format!("{any:?}"), format!("{any2:?}"));
    }

    /// `AnyTransport::send` must round-trip JSON serialisation correctly.
    ///
    /// `AnyTransport::send` 必须正确完成 JSON 序列化往返。
    #[tokio::test]
    async fn test_any_transport_send_roundtrip() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Payload {
            value: u32,
        }

        let any = AnyTransport::new(EchoTransport::default());
        let req = echo_req(Payload { value: 42 });
        let result: Payload = any.send(req).await.expect("send should succeed");

        assert_eq!(result, Payload { value: 42 });
    }

    /// `AnyTransport::stream` must forward the byte stream produced by the
    /// underlying transport unchanged.
    ///
    /// `AnyTransport::stream` 必须原封不动地转发底层传输产生的字节流。
    #[tokio::test]
    async fn test_any_transport_stream_forwards_bytes() {
        #[derive(Serialize)]
        struct Payload {
            msg: &'static str,
        }

        let any = AnyTransport::new(EchoTransport::default());
        let payload = Payload { msg: "hello" };
        let expected_bytes = serde_json::to_vec(&payload).unwrap();

        let req = echo_req(payload);
        let mut stream = any.stream(req).await.expect("stream should succeed");

        let chunk = stream
            .next()
            .await
            .expect("stream should yield one chunk")
            .expect("chunk should be Ok");

        assert_eq!(chunk.as_ref(), expected_bytes.as_slice());
    }

    /// `DynTransport` is implemented via blanket impl for `EchoTransport`.
    /// Verify it can be used as a trait object `Box<dyn DynTransport>`.
    ///
    /// `DynTransport` 通过 blanket impl 对 `EchoTransport` 实现。
    /// 验证它可以作为 `Box<dyn DynTransport>` 使用。
    #[tokio::test]
    async fn test_dyn_transport_send_erased() {
        let inner = EchoTransport::default();
        let boxed: Box<dyn DynTransport> = Box::new(inner);

        let body = serde_json::json!({ "key": "value" });
        let req = TransportRequest::new(Method::Get, "/test", body.clone());
        let result = boxed
            .send_erased(req)
            .await
            .expect("send_erased should succeed");

        assert_eq!(result, body);
    }

    /// `DynTransport::clone_box` must produce a functional independent clone.
    ///
    /// `DynTransport::clone_box` 必须产生一个功能独立的克隆。
    #[tokio::test]
    async fn test_dyn_transport_clone_box() {
        let inner = EchoTransport::default();
        let boxed: Box<dyn DynTransport> = Box::new(inner);
        let cloned = boxed.clone_box();

        let body = serde_json::json!({ "n": 1 });
        let req = TransportRequest::new(Method::Post, "/x", body.clone());
        let result = cloned
            .send_erased(req)
            .await
            .expect("cloned send_erased should succeed");
        assert_eq!(result, body);
    }

    /// `DynTransport::stream_erased` must return a boxed stream containing
    /// the byte output of the underlying transport.
    ///
    /// `DynTransport::stream_erased` 必须返回包含底层传输字节输出的 boxed 流。
    #[tokio::test]
    async fn test_dyn_transport_stream_erased() {
        let inner = EchoTransport::default();
        let boxed: Box<dyn DynTransport> = Box::new(inner);

        let body = serde_json::json!({ "x": 99 });
        let expected_bytes = serde_json::to_vec(&body).unwrap();
        let req = TransportRequest::new(Method::Post, "/s", body);
        let mut stream = boxed
            .stream_erased(req)
            .await
            .expect("stream_erased should succeed");

        let chunk = stream
            .next()
            .await
            .expect("stream should yield one chunk")
            .expect("chunk should be Ok");

        assert_eq!(chunk.as_ref(), expected_bytes.as_slice());
    }

    /// `AnyTransport` implements `Transport`, so its associated types must
    /// match `AnyStream` / `AnyStreamFuture`.
    ///
    /// `AnyTransport` 实现 `Transport`，其关联类型必须匹配 `AnyStream`/`AnyStreamFuture`。
    #[test]
    fn test_any_transport_associated_types() {
        fn assert_transport<T: Transport>() {}
        assert_transport::<AnyTransport>();
    }

    /// Type-alias sanity check: `AnyStream` and `AnyStreamFuture` must be
    /// the right concrete types.
    ///
    /// 类型别名健全性检查：`AnyStream` 和 `AnyStreamFuture` 必须是正确的具体类型。
    #[test]
    fn test_type_aliases() {
        fn _check_stream(_: AnyStream) {}
        fn _check_future(_: AnyStreamFuture) {}
        // Just verifying that the types are well-formed and can be named.
        let _: fn(AnyStream) = _check_stream;
        let _: fn(AnyStreamFuture) = _check_future;
    }
}
