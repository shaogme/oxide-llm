use super::{Content, HarmCategory, HarmProbability, UsageMetadata};
use serde::{Deserialize, Serialize};

/// Response from the model supporting multiple candidate responses.
///
/// 支持多个候选项响应的模型响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponse {
    /// Candidate responses from the model.
    ///
    /// 模型的候选项响应。
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    /// Returns the prompt's feedback related to the content filters.
    ///
    /// 返回与内容过滤器相关的提示反馈。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_feedback: Option<PromptFeedback>,
    /// Output only. Metadata on the generation requests' token usage.
    ///
    /// 仅输出。关于生成请求的令牌使用的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
    /// Output only. The model version used to generate the response.
    ///
    /// 仅输出。用于生成响应的模型版本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Output only. responseId is used to identify each response.
    ///
    /// 仅输出。responseId 用于标识每个响应。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    /// Output only. The current model status of this model.
    ///
    /// 仅输出。此模型的当前模型状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_status: Option<ModelStatus>,
}

/// A response candidate generated from the model.
///
/// 模型生成的响应候选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// Output only. Generated content returned from the model.
    ///
    /// 仅输出。模型返回的生成内容。
    pub content: Content,
    /// Optional. Output only. The reason why the model stopped generating tokens.
    ///
    /// 可选。仅输出。模型停止生成令牌的原因。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// List of ratings for the safety of a response candidate.
    ///
    /// 响应候选项的安全评级列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    /// Output only. Citation information for model-generated candidate.
    ///
    /// 仅输出。模型生成的候选项的引用信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,
    /// Output only. Token count for this candidate.
    ///
    /// 仅输出。此候选项的令牌计数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<i32>,
    /// Output only. Attribution information for sources that contributed to a grounded answer.
    ///
    /// 仅输出。对有依据的答案有贡献的来源的归因信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_attributions: Option<Vec<GroundingAttribution>>,
    /// Output only. Grounding metadata for the candidate.
    ///
    /// 仅输出。候选项的依据元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,
    /// Output only. Average log probability score of the candidate.
    ///
    /// 仅输出。候选项的平均对数概率得分。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f64>,
    /// Output only. Log-likelihood scores for the response tokens and top tokens.
    ///
    /// 仅输出。响应令牌和顶部令牌的对数似然分数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs_result: Option<LogprobsResult>,
    /// Output only. Output only. Index of the candidate in the list of response candidates.
    ///
    /// 仅输出。仅输出。响应候选项列表中候选项的索引。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    /// Optional. Output only. Details the reason why the model stopped generating tokens.
    ///
    /// 可选。仅输出。详细说明模型停止生成令牌的原因。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_message: Option<String>,
    /// Optional. Output only. Metadata related to url context retrieval tool.
    ///
    /// 可选。仅输出。与 URL 上下文检索工具相关的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context_metadata: Option<UrlContextMetadata>,
}

/// Defines the reason why the model stopped generating tokens.
///
/// 定义模型停止生成令牌的原因。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    #[serde(alias = "")]
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Language,
    Other,
    Blocklist,
    ProhibitedContent,
    Spii,
    FunctionCall,
    MalformedFunctionCall,
    ImageSafety,
    ImageProhibitedContent,
    ImageOther,
    NoImage,
    ImageRecitation,
    UnexpectedToolCall,
    TooManyToolCalls,
    MissingThoughtSignature,
    MalformedResponse,
    Escalation,
}

/// A set of the feedback metadata the prompt specified in `GenerateContentRequest.content`.
///
/// `GenerateContentRequest.content` 中指定的提示的反馈元数据集。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    /// Optional. If set, the prompt was blocked and no candidates are returned.
    ///
    /// 可选。如果设置，则提示被阻止，并且不返回任何候选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<BlockReason>,
    /// Ratings for safety of the prompt.
    ///
    /// 提示的安全性评级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Specifies the reason why the prompt was blocked.
///
/// 指定提示被阻止的原因。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockReason {
    BlockReasonUnspecified,
    Safety,
    Other,
    Blocklist,
    ProhibitedContent,
    ImageSafety,
}

/// Safety rating for a piece of content.
///
/// 一条内容的安全评级。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRating {
    /// Required. The category for this rating.
    ///
    /// 必填。此评级的类别。
    pub category: HarmCategory,
    /// Required. The probability of harm for this content.
    ///
    /// 必填。此内容有害的概率。
    pub probability: HarmProbability,
    /// Was this content blocked because of this rating?
    ///
    /// 此内容是否因为此评级而被阻止？
    #[serde(default)]
    pub blocked: bool,
}

/// A collection of source attributions for a piece of content.
///
/// 一条内容的来源归因集合。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    /// Citations to sources for a specific response.
    ///
    /// 针对特定响应的来源引用。
    #[serde(default)]
    pub citation_sources: Vec<CitationSource>,
}

/// A citation to a source for a portion of a specific response.
///
/// 对特定响应的一部分的来源引用。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    /// Optional. Start of segment of the response that is attributed to this source.
    ///
    /// 可选。归因于此源的响应段的开始。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// Optional. End of the attributed segment, exclusive.
    ///
    /// 可选。归因段的结束，不包含。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    /// Optional. URI that is attributed as a source for a portion of the text.
    ///
    /// 可选。归因于文本一部分的来源的 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional. License for the GitHub project that is attributed as a source for segment.
    ///
    /// 可选。归因为段来源的 GitHub 项目的许可证。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Attribution for a source that contributed to an answer.
///
/// 对答案有贡献的来源的归因。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingAttribution {
    /// Output only. Identifier for the source contributing to this attribution.
    ///
    /// 仅输出。标识对此归因有贡献的来源。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<AttributionSourceId>,
    /// Grounding source content that makes up this attribution.
    ///
    /// 构成此归因的依据源内容。
    pub content: Content,
}

/// Identifier for the source contributing to this attribution.
///
/// 标识对此归因有贡献的来源。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributionSourceId {
    /// Identifier for an inline passage.
    ///
    /// 内联段落的标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_passage: Option<GroundingPassageId>,
    /// Identifier for a `Chunk` fetched via Semantic Retriever.
    ///
    /// 通过语义检索器获取的 `Chunk` 的标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_retriever_chunk: Option<SemanticRetrieverChunk>,
}

/// Identifier for a part within a `GroundingPassage`.
///
/// `GroundingPassage` 中一部分的标识符。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingPassageId {
    /// Output only. ID of the passage.
    ///
    /// 仅输出。段落的 ID。
    pub passage_id: String,
    /// Output only. Index of the part within the content.
    ///
    /// 仅输出。内容中部分的索引。
    pub part_index: i32,
}

/// Identifier for a `Chunk` retrieved via Semantic Retriever.
///
/// 通过语义检索器检索的 `Chunk` 的标识符。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRetrieverChunk {
    /// Output only. Name of the source.
    ///
    /// 仅输出。来源名称。
    pub source: String,
    /// Output only. Name of the `Chunk` containing the attributed text.
    ///
    /// 仅输出。包含归因文本的 `Chunk` 的名称。
    pub chunk: String,
}

/// Metadata returned to client when grounding is enabled.
///
/// 启用归因时返回给客户端的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    /// List of supporting references retrieved from specified grounding source.
    ///
    /// 从指定依据源检索的支持参考列表。
    #[serde(default)]
    pub grounding_chunks: Vec<GroundingChunk>,
    /// List of grounding support.
    ///
    /// 依据支持列表。
    #[serde(default)]
    pub grounding_supports: Vec<GroundingSupport>,
    /// Web search queries for the following-up web search.
    ///
    /// 后续网络搜索的网络搜索查询。
    #[serde(default)]
    pub web_search_queries: Vec<String>,
    /// Image search queries used for grounding.
    ///
    /// 用于依据的图像搜索查询。
    #[serde(default)]
    pub image_search_queries: Vec<String>,
    /// Optional. Google search entry for the following-up web searches.
    ///
    /// 可选。后续网络搜索的 Google 搜索条目。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_entry_point: Option<SearchEntryPoint>,
    /// Metadata related to retrieval in the grounding flow.
    ///
    /// 与归因流程中的检索相关的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_metadata: Option<RetrievalMetadata>,
    /// Optional. Resource name of the Google Maps widget context token.
    ///
    /// 可选。Google 地图小部件上下文令牌的资源名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps_widget_context_token: Option<String>,
}

/// Grounding chunk.
///
/// 依据块。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk {
    /// Grounding chunk from the web.
    ///
    /// 来自网络的依据块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<Web>,
    /// Optional. Grounding chunk from image search.
    ///
    /// 可选。图像搜索的依据块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<GroundingChunkImage>,
    /// Optional. Grounding chunk from context retrieved by the file search tool.
    ///
    /// 可选。文件搜索工具检索的上下文中的依据块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_context: Option<RetrievedContext>,
    /// Optional. Grounding chunk from Google Maps.
    ///
    /// 可选。来自 Google 地图的依据块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maps: Option<Maps>,
}

/// Grounding chunk from image search.
///
/// 图像搜索的依据块。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunkImage {
    /// The web page URI for attribution.
    ///
    /// 归因的网页 URI。
    pub source_uri: String,
    /// The image asset URL.
    ///
    /// 图像资源 URL。
    pub image_uri: String,
    /// The title of the web page that the image is from.
    ///
    /// 图像来源网页的标题。
    pub title: String,
    /// The root domain of the web page that the image is from.
    ///
    /// 图像来源网页的根域名。
    pub domain: String,
}

/// Chunk from the web.
///
/// 来自网络的块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web {
    /// URI reference of the chunk.
    ///
    /// 块的 URI 引用。
    pub uri: String,
    /// Title of the chunk.
    ///
    /// 块的标题。
    pub title: String,
}

/// Chunk from context retrieved by the file search tool.
///
/// 文件搜索工具检索的上下文中的块。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedContext {
    /// Optional. User-provided metadata about the retrieved context.
    ///
    /// 可选。用户提供的关于检索上下文的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,
    /// Optional. URI reference of the semantic retrieval document.
    ///
    /// 可选。语义检索文档的 URI 引用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional. Title of the document.
    ///
    /// 可选。文档标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional. Text of the chunk.
    ///
    /// 可选。块的文本。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional. Name of the FileSearchStore containing the document.
    ///
    /// 可选。包含该文档的 FileSearchStore 名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search_store: Option<String>,
    /// Optional. Page number of the retrieved context.
    ///
    /// 可选。检索上下文的页码。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<i32>,
    /// Optional. The media blob resource name for multimodal file search results.
    ///
    /// 可选。多模态文件搜索结果的媒体 Blob 资源结构。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
}

/// User provided metadata about the GroundingFact.
///
/// 关于 GroundingFact 的用户提供元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMetadata {
    /// The key of the metadata.
    ///
    /// 元数据的键。
    pub key: String,
    /// Optional. The string value of the metadata.
    ///
    /// 可选。元数据的字符串值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    /// Optional. A list of string values for the metadata.
    ///
    /// 可选。元数据的字符串值列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_list_value: Option<StringList>,
    /// Optional. The numeric value of the metadata.
    ///
    /// 可选。元数据的数值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<f64>,
}

/// A list of string values.
///
/// 字符串值列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringList {
    /// The string values of the list.
    ///
    /// 列表的字符串值。
    #[serde(default)]
    pub values: Vec<String>,
}

/// A grounding chunk from Google Maps.
///
/// 来自 Google 地图的依据块。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maps {
    /// URI reference of the place.
    ///
    /// 地点的 URI 引用。
    pub uri: String,
    /// Title of the place.
    ///
    /// 地点标题。
    pub title: String,
    /// Text description of the place answer.
    ///
    /// 地点答案的文本描述。
    pub text: String,
    /// This ID of the place.
    ///
    /// 地点的 ID。
    pub place_id: String,
    /// Sources that provide answers about the features of a given place in Google Maps.
    ///
    /// 提供有关 Google 地图中给定地点特征的答案的来源。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_answer_sources: Option<PlaceAnswerSources>,
}

/// Collection of sources that provide answers about the features of a given place in Google Maps.
///
/// 提供有关 Google 地图中给定地点特征的答案的来源集合。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceAnswerSources {
    /// Snippets of reviews that are used to generate answers.
    ///
    /// 用于生成答案的评论片段。
    #[serde(default)]
    pub review_snippets: Vec<ReviewSnippet>,
}

/// Encapsulates a snippet of a user review.
///
/// 封装用户评论的片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSnippet {
    /// The ID of the review snippet.
    ///
    /// 评论片段的 ID。
    pub review_id: String,
    /// A link that corresponds to the user review on Google Maps.
    ///
    /// 对应于 Google 地图上用户评论的链接。
    pub google_maps_uri: String,
    /// Title of the review.
    ///
    /// 评论的标题。
    pub title: String,
}

/// Grounding support.
///
/// 依据支持。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport {
    /// Optional. A list of indices specifying the citations associated with the claim.
    ///
    /// 可选。指定与声明相关的引用的索引列表。
    #[serde(default)]
    pub grounding_chunk_indices: Vec<i32>,
    /// Optional. Confidence score of the support references.
    ///
    /// 可选。支持参考的置信度分数。
    #[serde(default)]
    pub confidence_scores: Vec<f64>,
    /// Output only. Indices into the parts field of the candidate's content.
    ///
    /// 仅输出。候选项内容的 parts 字段中的索引。
    #[serde(default)]
    pub rendered_parts: Vec<i32>,
    /// Segment of the content this support belongs to.
    ///
    /// 此支持所属的内容段。
    pub segment: Segment,
}

/// Segment of the content.
///
/// 内容段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    /// The index of a Part object within its parent Content object.
    ///
    /// Part 对象在其父 Content 对象中的索引。
    pub part_index: i32,
    /// Start index in the given Part.
    ///
    /// 给定 Part 中的起始索引。
    pub start_index: i32,
    /// End index in the given Part.
    ///
    /// 给定 Part 中的结束索引。
    pub end_index: i32,
    /// The text corresponding to the segment from the response.
    ///
    /// 响应中对应于该段的文本。
    pub text: String,
}

/// Google search entry point.
///
/// Google 搜索入口点。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint {
    /// Optional. Web content snippet.
    ///
    /// 可选。Web 内容片段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_content: Option<String>,
    /// Optional. Base64 encoded JSON representing array of <search term, search url> tuple.
    ///
    /// 可选。Base64 编码的 JSON，表示 <搜索词, 搜索 URL> 元组的数组。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_blob: Option<String>,
}

/// Metadata related to retrieval in the grounding flow.
///
/// 与归因流程中的检索相关的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    /// Optional. Score indicating how likely information from google search could help answer the prompt.
    ///
    /// 可选。指示来自 Google 搜索的信息帮助回答提示的可能性有多大的分数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_dynamic_retrieval_score: Option<f64>,
}

/// Logprobs Result.
///
/// Logprobs 结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsResult {
    /// Length = total number of decoding steps.
    ///
    /// 长度 = 解码步骤的总数。
    #[serde(default)]
    pub top_candidates: Vec<TopCandidates>,
    /// Length = total number of decoding steps.
    ///
    /// 长度 = 解码步骤的总数。
    #[serde(default)]
    pub chosen_candidates: Vec<LogprobsCandidate>,
    /// Sum of log probabilities for all tokens.
    ///
    /// 所有令牌的对数概率之和。
    #[serde(default)]
    pub log_probability_sum: f64,
}

/// Candidates with top log probabilities at each decoding step.
///
/// 每个解码步骤中具有最高对数概率的候选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCandidates {
    /// Sorted by log probability in descending order.
    ///
    /// 按对数概率降序排列。
    #[serde(default)]
    pub candidates: Vec<LogprobsCandidate>,
}

/// Candidate for the logprobs token and score.
///
/// logprobs 令牌和分数的候选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsCandidate {
    /// The candidate's token string value.
    ///
    /// 候选项的令牌字符串值。
    pub token: String,
    /// The candidate's token id value.
    ///
    /// 候选项的令牌 ID 值。
    pub token_id: i32,
    /// The candidate's log probability.
    ///
    /// 候选项的对数概率。
    pub log_probability: f64,
}

/// The status of the underlying model.
///
/// 底层模型的状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    /// The stage of the underlying model.
    ///
    /// 底层模型的阶段。
    pub model_stage: ModelStage,
    /// The time at which the model will be retired.
    ///
    /// 模型将退役的时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retirement_time: Option<String>,
    /// A message explaining the model status.
    ///
    /// 解释模型状态的消息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Defines the stage of the underlying model.
///
/// 定义底层模型的阶段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelStage {
    ModelStageUnspecified,
    UnstableExperimental,
    Experimental,
    Preview,
    Stable,
    Legacy,
    Deprecated,
    Retired,
}

/// Metadata related to url context retrieval tool.
///
/// 与 URL 上下文检索工具相关的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlContextMetadata {
    /// List of url context.
    ///
    /// URL 上下文列表。
    #[serde(default)]
    pub url_metadata: Vec<UrlMetadata>,
}

/// Context of the a single url retrieval.
///
/// 单个 URL 检索的上下文。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlMetadata {
    /// Retrieved url by the tool.
    ///
    /// 工具检索到的 URL。
    pub retrieved_url: String,
    /// Status of the url retrieval.
    ///
    /// URL 检索的状态。
    pub url_retrieval_status: UrlRetrievalStatus,
}

/// Status of the url retrieval.
///
/// URL 检索的状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UrlRetrievalStatus {
    UrlRetrievalStatusUnspecified,
    UrlRetrievalStatusSuccess,
    UrlRetrievalStatusError,
    UrlRetrievalStatusPaywall,
    UrlRetrievalStatusUnsafe,
}
