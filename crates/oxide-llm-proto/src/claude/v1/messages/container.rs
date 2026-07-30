use serde::{Deserialize, Serialize};

/// Container details specified in request/response.
///
/// 请求或响应中指定的容器信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container identifier string.
    ///
    /// 容器标识符。
    pub id: String,

    /// Expiration time string (RFC 3339).
    ///
    /// 容器过期时间。
    pub expires_at: String,
}
