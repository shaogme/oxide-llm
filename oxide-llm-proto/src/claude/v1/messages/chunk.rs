use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::response::{MessagesResponse, StopReason};
use super::{
    ImageBlock, RedactedThinkingBlock, SearchResultBlock, ServerToolUseBlock, TextBlock,
    ToolUseBlock, WebSearchToolResultBlock,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageStreamEvent {
    MessageStart {
        message: MessagesResponse,
    },
    ContentBlockStart {
        index: u32,
        content_block: ChunkContentBlock,
    },
    Ping,
    ContentBlockDelta {
        index: u32,
        delta: ChunkContentBlockDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: ChunkMessageDelta,
        usage: ChunkMessageDeltaUsage,
    },
    MessageStop,
    Error {
        error: ChunkError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkContentBlock {
    Text(TextBlock),
    Image(ImageBlock),
    ToolUse(ToolUseBlock),
    Thinking(ChunkThinkingBlock),
    RedactedThinking(RedactedThinkingBlock),
    #[serde(rename = "search_result")]
    SearchResult(SearchResultBlock),
    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlock),
    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkThinkingBlock {
    pub thinking: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkContentBlockDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: StaticRefStr },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: StaticRefStr },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<StaticRefStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessageDeltaUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkError {
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
    pub message: StaticRefStr,
}
