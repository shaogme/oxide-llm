use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::response::MediaResolution;

/// Content variant in Gemini Interactions API.
///
/// Gemini Interactions API 中的内容变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Text content block.
    ///
    /// 文本内容块。
    #[serde(rename = "text")]
    Text(TextContent),
    /// Image content block.
    ///
    /// 图像内容块。
    #[serde(rename = "image")]
    Image(ImageContent),
    /// Audio content block.
    ///
    /// 音频内容块。
    #[serde(rename = "audio")]
    Audio(AudioContent),
    /// Video content block.
    ///
    /// 视频内容块。
    #[serde(rename = "video")]
    Video(VideoContent),
    /// Document content block.
    ///
    /// 文档内容块。
    #[serde(rename = "document")]
    Document(DocumentContent),
}

/// A text content block.
///
/// 文本内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    /// The text content.
    ///
    /// 文本内容。
    pub text: String,
    /// Citation information for model-generated content.
    ///
    /// 模型生成内容的引用信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Annotation>>,
}

/// An image content block.
///
/// 图像内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    /// The base64 encoded image data.
    ///
    /// Base64 编码的图像数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// The mime type of the image.
    ///
    /// 图像的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// The resolution of the media.
    ///
    /// 媒体的分辨率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<MediaResolution>,
    /// The URI of the image.
    ///
    /// 图像的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// An audio content block.
///
/// 音频内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioContent {
    /// The base64 encoded audio data.
    ///
    /// Base64 编码的音频数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// The mime type of the audio.
    ///
    /// 音频的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// The URI of the audio.
    ///
    /// 音频的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
    /// The number of audio channels.
    ///
    /// 音频通道数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<i32>,
    /// The sample rate of the audio.
    ///
    /// 音频采样率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
}

/// A video content block.
///
/// 视频内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    /// The base64 encoded video data.
    ///
    /// Base64 编码的视频数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// The mime type of the video.
    ///
    /// 视频的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// The resolution of the media.
    ///
    /// 媒体的分辨率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<MediaResolution>,
    /// The URI of the video.
    ///
    /// 视频的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// A document content block.
///
/// 文档内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// The base64 encoded document data.
    ///
    /// Base64 编码的文档数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StaticRefStr>,
    /// The mime type of the document.
    ///
    /// 文档的 MIME 类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<StaticRefStr>,
    /// The URI of the document.
    ///
    /// 文档的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StaticRefStr>,
}

/// Citation information for model-generated content.
///
/// 模型生成内容的引用信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    /// URL citation.
    ///
    /// URL 引用。
    #[serde(rename = "url_citation")]
    UrlCitation(UrlCitation),
    /// File citation.
    ///
    /// 文件引用。
    #[serde(rename = "file_citation")]
    FileCitation(FileCitation),
    /// Place citation.
    ///
    /// 地点引用。
    #[serde(rename = "place_citation")]
    PlaceCitation(PlaceCitation),
    /// Word level transcription info.
    ///
    /// 词级转录信息。
    #[serde(rename = "word_info")]
    WordInfo(WordInfo),
}

/// A URL citation annotation.
///
/// URL 引用标注。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitation {
    /// Start of segment of the response attributed to this source.
    ///
    /// 归因于此源的响应段的起始索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// End of the attributed segment, exclusive.
    ///
    /// 归因段的结束索引（不包含）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// The title of the URL.
    ///
    /// URL 的标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The URL.
    ///
    /// URL 链接。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A file citation annotation.
///
/// 文件引用标注。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCitation {
    /// Start of segment of the response attributed to this source.
    ///
    /// 归因于此源的响应段的起始索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// End of the attributed segment, exclusive.
    ///
    /// 归因段的结束索引（不包含）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// The URI of the file.
    ///
    /// 文件的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_uri: Option<String>,
    /// The name of the file.
    ///
    /// 文件名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// Media ID in case of image citations.
    ///
    /// 图像引用的媒体 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
    /// Page number of the cited document.
    ///
    /// 引用文档的页码。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<i32>,
    /// Source attributed for a portion of text.
    ///
    /// 文本一部分归因的来源。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// User provided metadata about retrieved context.
    ///
    /// 用户提供的关于检索上下文的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<HashMap<String, serde_json::Value>>,
}

/// A place citation annotation.
///
/// 地点引用标注。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceCitation {
    /// Start of segment of the response attributed to this source.
    ///
    /// 归因于此源的响应段的起始索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// End of the attributed segment, exclusive.
    ///
    /// 归因段的结束索引（不包含）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// Title of the place.
    ///
    /// 地点标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The ID of the place.
    ///
    /// 地点的 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    /// URI reference of the place.
    ///
    /// 地点的 URI 引用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Snippets of reviews.
    ///
    /// 评论片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_snippets: Option<Vec<ReviewSnippet>>,
}

/// Snippet of a user review.
///
/// 用户评论片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSnippet {
    /// Review ID.
    ///
    /// 评论 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_id: Option<String>,
    /// Google Maps URI link.
    ///
    /// Google 地图 URI 链接。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_uri: Option<String>,
    /// Review title.
    ///
    /// 评论标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Word-level ASR annotation for transcription output.
///
/// 转录输出的词级 ASR 标注。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// Start of segment index.
    ///
    /// 起始段索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// End of segment index.
    ///
    /// 结束段索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// Transcribed word text.
    ///
    /// 转录词文本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Speaker label.
    ///
    /// 说话人标签。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    /// Start offset in time.
    ///
    /// 起始时间偏移量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_offset: Option<String>,
    /// End offset in time.
    ///
    /// 结束时间偏移量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_offset: Option<String>,
}

/// A thought content block.
///
/// 思考内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtContent {
    /// Signature of thought content.
    ///
    /// 思考内容的签名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
    /// Summary of the thought.
    ///
    /// 思考的摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Vec<ThoughtSummaryContent>>,
}

/// Thought summary content variant (Text or Image).
///
/// 思考摘要内容变体（文本或图像）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThoughtSummaryContent {
    /// Image content in thought summary.
    ///
    /// 思考摘要中的图像内容。
    #[serde(rename = "image")]
    Image(ImageContent),
    /// Text content in thought summary.
    ///
    /// 思考摘要中的文本内容。
    #[serde(rename = "text")]
    Text(TextContent),
}
