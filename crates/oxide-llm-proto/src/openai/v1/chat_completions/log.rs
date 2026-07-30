use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Log probability information for the choice.
///
/// 选项的对数概率信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// A list of message content tokens with log probability information.
    ///
    /// 带有对数概率信息的消息内容词元列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<LogProbToken>>,

    /// A list of message refusal tokens with log probability information.
    ///
    /// 带有对数概率信息的消息拒绝词元列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<LogProbToken>>,
}

/// Token log probability information.
///
/// 词元的对数概率信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbToken {
    /// The token text.
    ///
    /// 词元文本。
    pub token: StaticRefStr,

    /// The log probability of this token.
    ///
    /// 该词元的对数概率。
    pub logprob: f32,

    /// A list of integers representing the UTF-8 bytes representation of the token.
    ///
    /// 表示词元的 UTF-8 字节数组。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// List of the most likely tokens and their log probability at this token position.
    ///
    /// 在该词元位置上最可能的词元及其对数概率列表。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_logprobs: Vec<LogProbToken>,
}
