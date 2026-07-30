use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Log probability information for output tokens in OpenAI Response API.
///
/// OpenAI Response API 中输出词元的对数概率信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// A list of message content tokens with log probability information.
    ///
    /// 包含对数概率信息的消息内容词元列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<LogProbToken>>,

    /// A list of refusal tokens with log probability information.
    ///
    /// 包含对数概率信息的拒答词元列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<LogProbToken>>,
}

/// Detailed log probability information for a single token.
///
/// 单个词元的对数概率详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbToken {
    /// The text token.
    ///
    /// 文本词元。
    pub token: StaticRefStr,

    /// The log probability of this token.
    ///
    /// 该词元的对数概率。
    pub logprob: f32,

    /// A list of integers representing the UTF-8 bytes representation of the token.
    ///
    /// 代表词元 UTF-8 字节表示的整数列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// List of the most likely tokens and their log probabilities at this position.
    ///
    /// 在该词元位置上最可能的词元及其对数概率列表。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_logprobs: Vec<LogProbToken>,
}
