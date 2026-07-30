use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Configuration for enabling/disabling citations on documents.
///
/// 文档引用的使能配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsConfig {
    /// Whether citations are enabled.
    ///
    /// 是否开启引用。
    pub enabled: bool,
}

/// Citation reference in response text blocks.
///
/// 响应文本块中的引用条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Citation {
    /// Citation by character location in plain text.
    ///
    /// 纯文本中的字符位置引用。
    CharLocation(CharLocationCitation),

    /// Citation by page number in PDF document.
    ///
    /// PDF 文档中的页码引用。
    PageLocation(PageLocationCitation),

    /// Citation by block range in content.
    ///
    /// 内容块中的块索引引用。
    ContentBlockLocation(ContentBlockLocationCitation),

    /// Citation by web search result location.
    ///
    /// Web 搜索结果位置引用。
    WebSearchResultLocation(WebSearchResultLocationCitation),

    /// Citation by search result location.
    ///
    /// 搜索结果位置引用。
    SearchResultLocation(SearchResultLocationCitation),
}

/// Character location citation details.
///
/// 字符位置引用详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharLocationCitation {
    /// Full cited text string.
    ///
    /// 被引用的完整文本。
    pub cited_text: String,

    /// Document index in request documents array.
    ///
    /// 请求中文档的索引。
    pub document_index: u32,

    /// Document title if available.
    ///
    /// 文档标题（如果有）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<StaticRefStr>,

    /// Start character index (0-based).
    ///
    /// 起始字符索引。
    pub start_char_index: u32,

    /// End character index (0-based, exclusive).
    ///
    /// 结束字符索引。
    pub end_char_index: u32,

    /// Optional file ID.
    ///
    /// 可选的文件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<StaticRefStr>,
}

/// Page location citation details.
///
/// 页码位置引用详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLocationCitation {
    /// Full cited text string.
    ///
    /// 被引用的完整文本。
    pub cited_text: String,

    /// Document index in request documents array.
    ///
    /// 请求中文档的索引。
    pub document_index: u32,

    /// Document title if available.
    ///
    /// 文档标题（如果有）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<StaticRefStr>,

    /// Start page number (1-based).
    ///
    /// 起始页码。
    pub start_page_number: u32,

    /// End page number (1-based, inclusive).
    ///
    /// 结束页码。
    pub end_page_number: u32,

    /// Optional file ID.
    ///
    /// 可选的文件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<StaticRefStr>,
}

/// Content block location citation details.
///
/// 内容块位置引用详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockLocationCitation {
    /// Full cited text string.
    ///
    /// 被引用的完整文本。
    pub cited_text: String,

    /// Document index in request documents array.
    ///
    /// 请求中文档的索引。
    pub document_index: u32,

    /// Document title if available.
    ///
    /// 文档标题（如果有）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<StaticRefStr>,

    /// Start block index (0-based).
    ///
    /// 起始块索引。
    pub start_block_index: u32,

    /// End block index (0-based, exclusive).
    ///
    /// 结束块索引。
    pub end_block_index: u32,

    /// Optional file ID.
    ///
    /// 可选的文件 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<StaticRefStr>,
}

/// Web search result location citation details.
///
/// Web 搜索结果位置引用详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultLocationCitation {
    /// Full cited text string.
    ///
    /// 被引用的完整文本。
    pub cited_text: String,

    /// Encrypted index identifier.
    ///
    /// 加密的索引标识。
    pub encrypted_index: StaticRefStr,

    /// Title of the search result.
    ///
    /// 搜索结果的标题。
    pub title: StaticRefStr,

    /// URL of the search result.
    ///
    /// 搜索结果的 URL。
    pub url: StaticRefStr,
}

/// Search result location citation details.
///
/// 搜索结果位置引用详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultLocationCitation {
    /// Full cited text string.
    ///
    /// 被引用的完整文本。
    pub cited_text: String,

    /// End block index.
    ///
    /// 结束块索引。
    pub end_block_index: u32,

    /// 0-based index of cited search result.
    ///
    /// 搜索结果的 0 纪元索引。
    pub search_result_index: u32,

    /// Source identifier string.
    ///
    /// 来源标识。
    pub source: StaticRefStr,

    /// Start block index.
    ///
    /// 起始块索引。
    pub start_block_index: u32,

    /// Title of the cited search result.
    ///
    /// 被引用的搜索结果标题。
    pub title: StaticRefStr,
}
