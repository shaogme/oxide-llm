use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{Stream, StreamExt, stream::BoxStream};
use oxide_llm_core::transport::{Method, Transport, TransportError, TransportRequest};
use reqwest::{Client, RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};

/// Transport implementation based on `reqwest`.
///
/// 基于 `reqwest` 的传输层实现。
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: Client,
}

impl ReqwestTransport {
    /// Creates a new `ReqwestTransport` instance.
    ///
    /// 创建一个新的 `ReqwestTransport` 实例。
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Creates a new `ReqwestTransport` with a custom `reqwest::Client`.
    ///
    /// 使用自定义的 `reqwest::Client` 创建 `ReqwestTransport`。
    pub fn new_with_client(client: Client) -> Self {
        Self { client }
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

        let mut builder = self.client.request(method, &req.endpoint);

        for (k, v) in req.headers {
            builder = builder.header(k, v);
        }

        builder = builder.json(&req.body);

        Ok(builder)
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

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
        if !status.is_success() {
            let message = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
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
                if !status.is_success() {
                    let message = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
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
