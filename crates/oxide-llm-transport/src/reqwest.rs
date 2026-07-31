use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Stream, StreamExt, stream::BoxStream};
use oxide_llm_core::transport::{
    Method, Transport, TransportBuilder, TransportConfig, TransportError, TransportRequest,
};
use reqwest::{Client, RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};
use tracing::{debug, warn};

// ────────────────────────────────────────────────────────────────────────────
// Builder
// ────────────────────────────────────────────────────────────────────────────

/// Builder for [`ReqwestTransport`].
///
/// [`ReqwestTransport`] 的构造器。
///
/// Common options (`base_url`, `api_key`) are provided via the blanket methods
/// from [`TransportBuilder`]; reqwest-specific options (custom [`Client`]) are
/// added directly here.
///
/// 通用选项（`base_url`、`api_key`）由 [`TransportBuilder`] 的 blanket 方法提供；
/// reqwest 专属选项（自定义 [`Client`]）直接定义在此处。
#[derive(Debug, Default)]
pub struct ReqwestTransportBuilder {
    config: TransportConfig,
    client: Option<Client>,
}

impl TransportBuilder for ReqwestTransportBuilder {
    fn transport_config_mut(&mut self) -> &mut TransportConfig {
        &mut self.config
    }
}

impl ReqwestTransportBuilder {
    /// Sets a custom [`reqwest::Client`].
    ///
    /// 设置自定义 [`reqwest::Client`]。
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Builds the [`ReqwestTransport`].
    ///
    /// Returns `Err` if `base_url` has not been provided.
    ///
    /// 构建 [`ReqwestTransport`]。
    ///
    /// 若未提供 `base_url` 则返回 `Err`。
    pub fn build(self) -> Result<ReqwestTransport, TransportError> {
        let TransportConfig { base_url, api_key } = self.config;
        let base_url = base_url.ok_or_else(|| TransportError::Other {
            message: "ReqwestTransport: `base_url` is required".to_string(),
        })?;
        Ok(ReqwestTransport {
            client: self.client.unwrap_or_default(),
            base_url,
            api_key,
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Transport
// ────────────────────────────────────────────────────────────────────────────

/// Transport implementation based on `reqwest`.
///
/// 基于 `reqwest` 的传输层实现。
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: Client,
    /// Base URL prepended to every relative endpoint.
    base_url: String,
    /// Optional Bearer-token API key.
    api_key: Option<String>,
}

impl ReqwestTransport {
    /// Returns a [`ReqwestTransportBuilder`] for fluent configuration.
    ///
    /// 返回用于链式配置的 [`ReqwestTransportBuilder`]。
    pub fn builder() -> ReqwestTransportBuilder {
        ReqwestTransportBuilder::default()
    }

    fn prepare_request<Req: Serialize>(
        &self,
        req: TransportRequest<Req>,
    ) -> Result<RequestBuilder, TransportError> {
        let method = match req.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        };

        // Resolve endpoint: prepend base_url when the endpoint is relative.
        let url = if req.endpoint.starts_with("http") {
            req.endpoint.into_owned()
        } else {
            let endpoint = req.endpoint.trim_start_matches('/');
            format!("{}/{}", self.base_url, endpoint)
        };

        debug!("Preparing HTTP request: {} {}", method, url);
        let mut builder = self.client.request(method, &url);

        // Inject Authorization header when an API key is configured.
        if let Some(ref key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        for (k, v) in req.headers {
            builder = builder.header(k.as_ref(), v.as_ref());
        }

        builder = builder.json(&req.body);

        Ok(builder)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Error mapping
// ────────────────────────────────────────────────────────────────────────────

fn map_reqwest_error(e: reqwest::Error) -> TransportError {
    let message = e.to_string();
    if e.is_decode() {
        TransportError::Codec { message }
    } else if e.is_timeout() {
        TransportError::Network {
            message: format!("Timeout: {}", message),
        }
    } else if e.is_connect() {
        TransportError::Network {
            message: format!("Connection error: {}", message),
        }
    } else if let Some(status) = e.status() {
        TransportError::Api {
            status: status.as_u16(),
            message,
        }
    } else {
        TransportError::Other { message }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Stream types
// ────────────────────────────────────────────────────────────────────────────

/// Stream implementation for `ReqwestTransport`.
///
/// `ReqwestTransport` 的流实现。
pub struct ReqwestStream {
    inner: BoxStream<'static, Result<bytes::Bytes, TransportError>>,
}

impl ReqwestStream {
    /// Creates a new `ReqwestStream` wrapping the inner stream.
    ///
    /// 创建包装内部流的新 `ReqwestStream`。
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    {
        Self {
            inner: stream
                .map(|chunk_result| chunk_result.map_err(map_reqwest_error))
                .boxed(),
        }
    }
}

impl Stream for ReqwestStream {
    type Item = Result<bytes::Bytes, TransportError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Future implementation for `ReqwestTransport::stream`.
///
/// `ReqwestTransport::stream` 的 Future 实现。
pub struct ReqwestStreamFuture {
    inner: futures::future::BoxFuture<'static, Result<ReqwestStream, TransportError>>,
}

impl std::future::Future for ReqwestStreamFuture {
    type Output = Result<ReqwestStream, TransportError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Transport impl
// ────────────────────────────────────────────────────────────────────────────

impl Transport for ReqwestTransport {
    type Stream = ReqwestStream;
    type StreamFuture = ReqwestStreamFuture;

    async fn send<Req, Res>(&self, req: TransportRequest<Req>) -> Result<Res, TransportError>
    where
        Req: Serialize + Send + Sync,
        Res: DeserializeOwned + Send + Sync,
    {
        let builder = self.prepare_request(req)?;
        let resp = builder.send().await.map_err(map_reqwest_error)?;

        let status = resp.status();
        debug!("HTTP response status: {}", status);
        if !status.is_success() {
            let message = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!("HTTP response error [{}]: {}", status.as_u16(), message);
            return Err(TransportError::Api {
                status: status.as_u16(),
                message,
            });
        }

        resp.json::<Res>().await.map_err(map_reqwest_error)
    }

    fn stream<Req>(&self, req: TransportRequest<Req>) -> Self::StreamFuture
    where
        Req: Serialize + Send + Sync + 'static,
    {
        let this = self.clone();
        ReqwestStreamFuture {
            inner: Box::pin(async move {
                let builder = this.prepare_request(req)?;
                let resp = builder.send().await.map_err(map_reqwest_error)?;

                let status = resp.status();
                debug!("HTTP stream response status: {}", status);
                if !status.is_success() {
                    let message = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    warn!(
                        "HTTP stream response error [{}]: {}",
                        status.as_u16(),
                        message
                    );
                    return Err(TransportError::Api {
                        status: status.as_u16(),
                        message,
                    });
                }

                Ok(ReqwestStream::new(resp.bytes_stream()))
            }),
        }
    }
}
