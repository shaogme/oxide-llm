use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A `Tool` is a piece of code that enables the system to interact with external systems.
///
/// `Tool` 原是一个代码片段，使系统能够与外部系统交互。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Optional. A list of `FunctionDeclarations` available to the `Model` that can be used for function calling.
    ///
    /// 可选。`Model` 可用于函数调用的 `FunctionDeclarations` 列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,
    /// Optional. GoogleSearchRetrieval tool type.
    ///
    /// 可选。GoogleSearchRetrieval 工具类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,
    /// Optional. CodeExecution tool type.
    ///
    /// 可选。CodeExecution 工具类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<CodeExecution>,
    /// Optional. GoogleSearch tool type.
    ///
    /// 可选。GoogleSearch 工具类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GoogleSearch>,
    /// Optional. Tool to support the model interacting directly with the computer.
    ///
    /// 可选。支持模型直接与计算机交互的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<ComputerUse>,
    /// Optional. Tool to support URL context retrieval.
    ///
    /// 可选。支持 URL 上下文检索的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,
    /// Optional. FileSearch tool type.
    ///
    /// 可选。FileSearch 工具类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<FileSearch>,
    /// Optional. MCP Servers to connect to.
    ///
    /// 可选。要连接的 MCP 服务器。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<McpServer>>,
    /// Optional. Tool that allows grounding the model's response with geospatial context.
    ///
    /// 可选。允许使用地理空间上下文建立模型响应的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps: Option<GoogleMaps>,
}

/// A MCPServer is a server that can be called by the model to perform actions.
///
/// MCPServer 是模型可以调用以执行操作的服务器。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// The name of the MCPServer.
    ///
    /// MCPServer 的名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// A transport that can stream HTTP requests and responses.
    ///
    /// 可以流式传输 HTTP 请求和响应的传输。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streamable_http_transport: Option<StreamableHttpTransport>,
}

/// A transport that can stream HTTP requests and responses.
///
/// 可以流式传输 HTTP 请求和响应的传输。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamableHttpTransport {
    /// Required. The full URL for the MCPServer endpoint.
    ///
    /// 必填。MCPServer 端点的完整 URL。
    pub url: StaticRefStr,
    /// Optional. Fields for authentication headers, timeouts, etc.
    ///
    /// 可选。身份验证标头、超时等的字段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<StaticRefStr, StaticRefStr>>,
    /// Optional. HTTP timeout for regular operations.
    ///
    /// 可选。常规操作的 HTTP 超时。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<StaticRefStr>,
    /// Optional. Timeout for SSE read operations.
    ///
    /// 可选。SSE 读取操作的超时。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sse_read_timeout: Option<StaticRefStr>,
    /// Optional. Whether to close the client session when the transport closes.
    ///
    /// 可选。传输关闭时是否关闭客户端会话。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_on_close: Option<bool>,
}

/// Structured representation of a function declaration.
///
/// 函数声明的结构化表示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
    /// Required. The name of the function.
    ///
    /// 必填。函数的名称。
    pub name: StaticRefStr,
    /// Required. A brief description of the function.
    ///
    /// 必填。函数的简要描述。
    pub description: StaticRefStr,
    /// Optional. Specifies the function Behavior.
    ///
    /// 可选。指定函数行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<Behavior>,
    /// Optional. Describes the parameters to this function.
    ///
    /// 可选。描述此函数的参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
    /// Optional. Describes the parameters to the function in JSON Schema format.
    ///
    /// 可选。以 JSON Schema 格式描述此函数的参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_json_schema: Option<serde_json::Value>,
    /// Optional. Describes the output from this function in JSON Schema format.
    ///
    /// 可选。以 JSON Schema 格式描述此函数的输出。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Schema>,
    /// Optional. Describes the output from this function in JSON Schema format.
    ///
    /// 可选。以 JSON Schema 格式描述此函数的输出。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<serde_json::Value>,
}

/// Defines the function behavior.
///
/// 定义函数行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Behavior {
    Unspecified,
    Blocking,
    NonBlocking,
}

/// Tool configuration for any `Tool` specified in the request.
///
/// 请求中指定的任何 `Tool` 的工具配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    /// Optional. Function calling config.
    ///
    /// 可选。函数调用配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_calling_config: Option<FunctionCallingConfig>,
    /// Optional. Retrieval config.
    ///
    /// 可选。检索配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_config: Option<RetrievalConfig>,
}

/// Configuration for function calling.
///
/// 函数调用配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallingConfig {
    /// Optional. Specifies the mode in which function calling should execute.
    ///
    /// 可选。指定函数调用应执行的模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FunctionCallingConfigMode>,
    /// Optional. A set of function names that, when provided, limits the functions the model will call.
    ///
    /// 可选。一组函数名称，如果提供，将限制模型将调用的函数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_function_names: Option<Vec<StaticRefStr>>,
}

/// Defines the execution mode for function calling.
///
/// 定义函数调用的执行模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionCallingConfigMode {
    ModeUnspecified,
    Auto,
    Any,
    None,
}

/// Tool to retrieve information from Google Search.
///
/// 从 Google 搜索检索信息的工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchRetrieval {
    /// Optional. Specifies the dynamic retrieval configuration for the given source.
    ///
    /// 可选。指定给定源的动态检索配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_retrieval_config: Option<DynamicRetrievalConfig>,
}

/// Describes the dynamic retrieval configuration for the given source.
///
/// 描述给定源的动态检索配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrievalConfig {
    /// Optional. The mode of the dynamic retrieval.
    ///
    /// 可选。动态检索的模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<DynamicRetrievalMode>,
    /// Optional. The threshold to be used for dynamic retrieval.
    ///
    /// 可选。用于动态检索的阈值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_threshold: Option<f32>,
}

/// The mode of the dynamic retrieval.
///
/// 动态检索的模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DynamicRetrievalMode {
    ModeUnspecified,
    ModeDynamic,
}

/// Tool that enables the model to execute code as part of generation.
///
/// 使模型能够作为生成的一部分执行代码的工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecution {}

/// GoogleSearch tool type.
///
/// GoogleSearch 工具类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearch {
    /// Optional. Filter search results to a specific time range.
    ///
    /// 可选。将搜索结果过滤到特定时间范围。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range_filter: Option<Interval>,
}

/// Represents a time interval.
///
/// 表示时间间隔。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interval {
    /// Optional. Inclusive start of the interval.
    ///
    /// 可选。间隔的起始（包含）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<StaticRefStr>,
    /// Optional. Exclusive end of the interval.
    ///
    /// 可选。间隔的结束（不包含）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<StaticRefStr>,
}

/// Computer Use tool type.
///
/// 计算机使用工具类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerUse {
    /// Required. The environment being operated.
    ///
    /// 必填。正在操作的环境。
    pub environment: Environment,
    /// Optional. By default, predefined functions are included in the final model call.
    ///
    /// 可选。默认情况下，预定义函数包含在最终模型调用中。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_predefined_functions: Option<Vec<StaticRefStr>>,
}

/// Represents the environment being operated, such as a web browser.
///
/// 表示正在操作的环境，例如 Web 浏览器。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Environment {
    EnvironmentUnspecified,
    EnvironmentBrowser,
}

/// Tool to support URL context retrieval.
///
/// 支持 URL 上下文检索的工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlContext {}

/// The FileSearch tool that retrieves knowledge from Semantic Retrieval corpora.
///
/// 从语义检索语料库检索知识的 FileSearch 工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearch {
    /// Required. The names of the fileSearchStores to retrieve from.
    ///
    /// 必填。要从中检索的 fileSearchStores 的名称。
    pub file_search_store_names: Vec<StaticRefStr>,
    /// Optional. Metadata filter to apply to the semantic retrieval documents and chunks.
    ///
    /// 可选。应用于语义检索文档和块的元数据过滤器。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<StaticRefStr>,
    /// Optional. The number of semantic retrieval chunks to retrieve.
    ///
    /// 可选。要检索的语义检索块的数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}

/// The GoogleMaps Tool that provides geospatial context for the user's query.
///
/// 为用户查询提供地理空间上下文的 GoogleMaps 工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleMaps {
    /// Optional. Whether to return a widget context token in the GroundingMetadata.
    ///
    /// 可选。是否在 GroundingMetadata 中返回小部件上下文令牌。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_widget: Option<bool>,
}

/// Retrieval config.
///
/// 检索配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalConfig {
    /// Optional. The location of the user.
    ///
    /// 可选。用户的位置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat_lng: Option<LatLng>,
    /// Optional. The language code of the user.
    ///
    /// 可选。用户的语言代码。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<StaticRefStr>,
}

/// An object that represents a latitude/longitude pair.
///
/// 表示纬度/经度对的对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatLng {
    /// The latitude in degrees.
    ///
    /// 纬度（度）。
    pub latitude: f64,
    /// The longitude in degrees.
    ///
    /// 经度（度）。
    pub longitude: f64,
}

/// The Schema object allows the definition of input and output data types.
/// These types can be objects, but also primitives and arrays.
/// Represents a subset of the [OpenAPI schema](https://spec.openapis.org/oas/v3.0.3#schema).
///
/// Schema 对象允许定义输入和输出数据类型。
/// 这些类型可以是对象，也可以是基元和数组。
/// 表示 [OpenAPI 模式](https://spec.openapis.org/oas/v3.0.3#schema) 的子集。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// Required. Data type.
    ///
    /// 必填。数据类型。
    #[serde(rename = "type")]
    pub schema_type: Type,
    /// Optional. The format of the data.
    ///
    /// 可选。数据的格式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StaticRefStr>,
    /// Optional. The title of the schema.
    ///
    /// 可选。架构的标题。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<StaticRefStr>,
    /// Optional. A brief description of the parameter.
    ///
    /// 可选。参数的简要说明。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,
    /// Optional. Indicates if the value may be null.
    ///
    /// 可选。指示该值是否可以为 null。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    /// Optional. Possible values of the element of Type.STRING with enum format.
    ///
    /// 可选。Type.STRING 类型的元素以及枚举格式的可能值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<StaticRefStr>>,
    /// Optional. Maximum number of the elements for Type.ARRAY.
    ///
    /// 可选。Type.ARRAY 元素的最大数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<StaticRefStr>,
    /// Optional. Minimum number of the elements for Type.ARRAY.
    ///
    /// 可选。Type.ARRAY 元素的最小数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<StaticRefStr>,
    /// Optional. Properties of Type.OBJECT.
    ///
    /// 可选。Type.OBJECT 的属性。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<StaticRefStr, Schema>>,
    /// Optional. Required properties of Type.OBJECT.
    ///
    /// 可选。Type.OBJECT 的必需属性。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<StaticRefStr>>,
    /// Optional. Minimum number of the properties for Type.OBJECT.
    ///
    /// 可选。Type.OBJECT 的最小属性数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<StaticRefStr>,
    /// Optional. Maximum number of the properties for Type.OBJECT.
    ///
    /// 可选。Type.OBJECT 的最大属性数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<StaticRefStr>,
    /// Optional. Minimum length of the Type.STRING.
    ///
    /// 可选。Type.STRING 的最小长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<StaticRefStr>,
    /// Optional. Maximum length of the Type.STRING.
    ///
    /// 可选。Type.STRING 的最大长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<StaticRefStr>,
    /// Optional. Pattern of the Type.STRING to restrict a string to a regular expression.
    ///
    /// 可选。Type.STRING 的正则表达式限制模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<StaticRefStr>,
    /// Optional. Example of the object.
    ///
    /// 可选。对象的示例。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// Optional. The value should be validated against any (one or more) of the subschemas.
    ///
    /// 可选。值应针对列表中的任何一个或多个子架构进行验证。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    /// Optional. The order of the properties.
    ///
    /// 可选。属性的顺序。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_ordering: Option<Vec<StaticRefStr>>,
    /// Optional. Default value of the field.
    ///
    /// 可选。字段的默认值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Optional. Schema of the elements of Type.ARRAY.
    ///
    /// 可选。Type.ARRAY 元素的架构。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    /// Optional. Minimum value of Type.INTEGER and Type.NUMBER.
    ///
    /// 可选。Type.INTEGER 和 Type.NUMBER 的最小值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Optional. Maximum value of Type.INTEGER and Type.NUMBER.
    ///
    /// 可选。Type.INTEGER 和 Type.NUMBER 的最大值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

/// Type contains the list of OpenAPI data types as defined by [OpenAPI schema](https://spec.openapis.org/oas/v3.0.3#schema).
///
/// Type 包含 [OpenAPI 模式](https://spec.openapis.org/oas/v3.0.3#schema) 定义的 OpenAPI 数据类型列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    TypeUnspecified,
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}
