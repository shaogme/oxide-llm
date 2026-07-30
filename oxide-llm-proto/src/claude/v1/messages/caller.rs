use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Caller information for tool invocations.
///
/// 工具调用的调用方信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Caller {
    /// Direct invocation by the model.
    ///
    /// 模型直接调用。
    Direct(DirectCaller),

    /// Server-side tool caller (2025-08-25).
    ///
    /// 服务端工具调用方 (2025-08-25)。
    Server(ServerToolCaller),

    /// Server-side tool caller (2026-01-20).
    ///
    /// 服务端工具调用方 (2026-01-20)。
    Server20260120(ServerToolCaller20260120),
}

/// Direct invocation caller.
///
/// 直接调用方。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectCaller {
    /// Type of caller ("direct").
    ///
    /// 调用方类型（固定为 "direct"）。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}

/// Server-side tool caller (2025-08-25).
///
/// 服务端工具调用方 (2025-08-25)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolCaller {
    /// ID of the generating tool.
    ///
    /// 生成该调用的工具 ID。
    pub tool_id: StaticRefStr,

    /// Type of caller ("code_execution_20250825").
    ///
    /// 调用方类型。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}

/// Server-side tool caller (2026-01-20).
///
/// 服务端工具调用方 (2026-01-20)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolCaller20260120 {
    /// ID of the generating tool.
    ///
    /// 生成该调用的工具 ID。
    pub tool_id: StaticRefStr,

    /// Type of caller ("code_execution_20260120").
    ///
    /// 调用方类型。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}
