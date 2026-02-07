use futures::{StreamExt, stream::BoxStream};
use oxide_llm_core::transport::{LocalTransport, Method, TransportError, TransportRequest};
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

impl LocalTransport for ReqwestTransport {
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

    async fn stream<Req>(
        &self,
        req: TransportRequest<Req>,
    ) -> Result<BoxStream<'static, Result<bytes::Bytes, TransportError>>, TransportError>
    where
        Req: Serialize + Send + Sync,
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

        let stream = resp
            .bytes_stream()
            .map(|chunk_result| chunk_result.map_err(map_reqwest_error));

        Ok(stream.boxed())
    }
}
