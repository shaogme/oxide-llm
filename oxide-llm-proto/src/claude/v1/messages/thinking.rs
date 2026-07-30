use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Thinking block containing model reasoning text and signature.
///
/// 包含模型思考推理过程与签名信息的 Thinking 块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    /// Signature for verifying thinking integrity.
    ///
    /// 验证思考完整性的签名。
    pub signature: StaticRefStr,

    /// Thinking content text.
    ///
    /// 思考文本内容。
    pub thinking: String,
}

/// Redacted thinking block.
///
/// 脱敏后的思考内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedThinkingBlock {
    /// Encrypted/redacted data string.
    ///
    /// 加密或脱敏的数据字符串。
    pub data: StaticRefStr,
}

/// Extended thinking configuration parameter.
///
/// 深度思考模式的配置参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfigParam {
    /// Thinking mode enabled.
    ///
    /// 启用思考模式。
    Enabled {
        /// Token budget dedicated to reasoning.
        ///
        /// 专门用于推理的 token 预算。
        budget_tokens: u32,

        /// Controls display mode ("summarized" or "omitted").
        ///
        /// 控制思考内容的显示方式（"summarized" 或 "omitted"）。
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },

    /// Thinking mode disabled.
    ///
    /// 禁用思考模式。
    Disabled,

    /// Adaptive thinking mode.
    ///
    /// 自适应思考模式。
    Adaptive {
        /// Controls display mode ("summarized" or "omitted").
        ///
        /// 控制思考内容的显示方式（"summarized" 或 "omitted"）。
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
}

/// Display mode for thinking content.
///
/// 思考内容的显示模式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingDisplay {
    /// Thinking returned normally.
    ///
    /// 正常返回思考内容。
    Summarized,

    /// Thinking content redacted.
    ///
    /// 思考内容脱敏隐去。
    Omitted,
}
