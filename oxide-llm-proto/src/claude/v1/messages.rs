use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod request;
pub mod response;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),
    Image(ImageBlock),
    ToolUse(ToolUseBlock),
    ToolResult(ToolResultBlock),
    Document(DocumentBlock),
    Thinking(ThinkingBlock),
    RedactedThinking(RedactedThinkingBlock),
    // New block types
    #[serde(rename = "search_result")]
    SearchResult(SearchResultBlock),
    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlock),
    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Citation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    Base64 {
        #[serde(rename = "type")]
        typ: String, // "base64"
        media_type: String,
        data: String,
    },
    Url {
        #[serde(rename = "type")]
        typ: String, // "url"
        url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: ToolResultContent, // string or blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ContentBlock>), // Usually Text or Image
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBlock {
    pub source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentSource {
    Base64 {
        #[serde(rename = "type")]
        typ: String, // "base64"
        media_type: String, // "application/pdf"
        data: String,
    },
    Url {
        #[serde(rename = "type")]
        typ: String, // "url"
        url: String,
    },
    Text {
        #[serde(rename = "type")]
        typ: String, // "text"
        media_type: String, // "text/plain"
        data: String,
    },
    Content {
        #[serde(rename = "type")]
        typ: String, // "content"
        content: Content, // string or blocks
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub signature: String,
    pub thinking: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedThinkingBlock {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub typ: String, // "ephemeral"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>, // "5m" or "1h"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Citation {
    CharLocation(CharLocationCitation),
    PageLocation(PageLocationCitation),
    ContentBlockLocation(ContentBlockLocationCitation),
    WebSearchResultLocation(WebSearchResultLocationCitation),
    SearchResultLocation(SearchResultLocationCitation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_char_index: u32,
    pub end_char_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_page_number: u32,
    pub end_page_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockLocationCitation {
    pub cited_text: String,
    pub document_index: u32,
    pub document_title: Option<String>,
    pub start_block_index: u32,
    pub end_block_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultLocationCitation {
    pub cited_text: String,
    pub encrypted_index: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultLocationCitation {
    pub cited_text: String,
    pub end_block_index: u32,
    pub search_result_index: u32,
    pub source: String,
    pub start_block_index: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultBlock {
    pub content: Vec<TextBlock>,
    pub source: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlock {
    pub id: String,
    pub name: String, // "web_search"
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlock {
    pub tool_use_id: String,
    pub content: Vec<WebSearchResultItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSearchResultItem {
    WebSearchResult(WebSearchResult),
    WebSearchToolResultError(WebSearchToolResultError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub encrypted_content: String,
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_age: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultError {
    pub error_code: String,
}
