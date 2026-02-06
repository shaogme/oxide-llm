use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::ContentPart;
use oxide_llm_proto::{
    claude::v1::messages::request::{
        CustomTool as ClaudeCustomTool, Tool as ClaudeTool, ToolChoice as ClaudeToolChoice,
    },
    gemini::v1beta::{
        FunctionCallingConfig as GeminiFunctionCallingConfig,
        FunctionCallingConfigMode as GeminiFunctionCallingConfigMode,
        FunctionDeclaration as GeminiFunctionDeclaration, ToolConfig as GeminiToolConfig,
    },
    openai::v1::{
        FunctionDefinition as OpenAIFunctionDefinition, Tool as OpenAITool,
        ToolChoice as OpenAIToolChoice, ToolChoiceFunction as OpenAIToolChoiceFunction,
        ToolChoiceNamed as OpenAIToolChoiceNamed,
    },
};

/// Universal Tool Definition.
///
/// Designed to cover OpenAI `Tool`, Gemini `FunctionDeclaration` (inside Tool), and Claude `Tool`.
///
/// 通用工具定义。
/// 旨在覆盖 OpenAI `Tool`，Gemini `FunctionDeclaration` (在 Tool 内部)，以及 Claude `Tool`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    pub r#type: ToolType,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Function,
    // Future extensions: CodeInterpreter, Retrieval, etc.
}

/// Function definition details.
///
/// 函数定义详情。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema parameters definition.
    ///
    /// JSON Schema 参数定义。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<JSONSchema>,

    /// Strict mode (OpenAI specific, but good to preserve).
    ///
    /// 严格模式 (OpenAI 特有，但在核心层保留有助于兼容)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// A strongly-typed JSON Schema definition for tool parameters.
///
/// 用于工具参数的强类型 JSON Schema 定义。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JSONSchema {
    /// The data type of the schema.
    ///
    /// 数据类型。
    pub schema_type: Option<JSONSchemaType>,

    /// A description of the valid data.
    ///
    /// 数据描述。
    pub description: Option<String>,

    /// Object properties (if type is object).
    /// Using BTreeMap for deterministic serialization order.
    ///
    /// 对象属性（如果类型是 object）。
    /// 使用 BTreeMap 以保证序列化顺序确定性。
    pub properties: Option<std::collections::BTreeMap<String, JSONSchema>>,

    /// List of required property names.
    ///
    /// 必须的属性名称列表。
    pub required: Option<Vec<String>>,

    /// Schema for array items (if type is array).
    ///
    /// 数组元素的 Schema（如果类型是 array）。
    pub items: Option<Box<JSONSchema>>,

    /// Enumeration of allowed values.
    ///
    /// 允许值的枚举。
    pub enum_values: Option<Vec<String>>,

    /// Additional properties allowance.
    /// Important for OpenAI Strict Mode (must be false).
    ///
    /// 是否允许额外属性。
    /// 对于 OpenAI 严格模式很重要（必须为 false）。
    pub additional_properties: Option<bool>,

    /// Format string (e.g., "date-time", "uri").
    ///
    /// 格式字符串。
    pub format: Option<String>,

    /// Default value.
    ///
    /// 默认值。
    pub default: Option<serde_json::Value>,

    /// Nullable flag.
    ///
    /// The source of truth for nullability in this internal representation.
    /// - When converting TO Gemini: maps to `nullable` field.
    /// - When converting TO OpenAI/Claude (standard JSON Schema): transforms `type` into `["type", "null"]` during serialization.
    pub nullable: Option<bool>,
}

/// JSON Schema data types.
///
/// JSON Schema 数据类型。
#[derive(Debug, Clone, PartialEq)]
pub enum JSONSchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Null,
}

// Custom Serialize/Deserialize for JSONSchemaType to handle simple strings or arrays (for standard JSON schema compatibility in raw JSON)
// However, since we are doing custom serialization on the parent struct, we might keep this simple.
impl Serialize for JSONSchemaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JSONSchemaType::String => serializer.serialize_str("string"),
            JSONSchemaType::Number => serializer.serialize_str("number"),
            JSONSchemaType::Integer => serializer.serialize_str("integer"),
            JSONSchemaType::Boolean => serializer.serialize_str("boolean"),
            JSONSchemaType::Object => serializer.serialize_str("object"),
            JSONSchemaType::Array => serializer.serialize_str("array"),
            JSONSchemaType::Null => serializer.serialize_str("null"),
        }
    }
}

impl<'de> Deserialize<'de> for JSONSchemaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "string" => Ok(JSONSchemaType::String),
            "number" => Ok(JSONSchemaType::Number),
            "integer" => Ok(JSONSchemaType::Integer),
            "boolean" => Ok(JSONSchemaType::Boolean),
            "object" => Ok(JSONSchemaType::Object),
            "array" => Ok(JSONSchemaType::Array),
            "null" => Ok(JSONSchemaType::Null),
            _ => Err(serde::de::Error::custom(format!("Unknown type: {}", s))),
        }
    }
}

// Custom Serialize for JSONSchema to handle "type": ["string", "null"] when nullable is true
impl Serialize for JSONSchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        // Handle Type & Nullable
        if let Some(ref t) = self.schema_type {
            if self.nullable == Some(true) && *t != JSONSchemaType::Null {
                // Serialize as array: ["type", "null"]
                let types = vec![t.clone(), JSONSchemaType::Null];
                map.serialize_entry("type", &types)?;
            } else {
                // Standard single type
                map.serialize_entry("type", t)?;
            }
        }

        if let Some(ref v) = self.description {
            map.serialize_entry("description", v)?;
        }
        if let Some(ref v) = self.properties {
            map.serialize_entry("properties", v)?;
        }
        if let Some(ref v) = self.required {
            map.serialize_entry("required", v)?;
        }
        if let Some(ref v) = self.items {
            map.serialize_entry("items", v)?;
        }
        if let Some(ref v) = self.enum_values {
            map.serialize_entry("enum", v)?;
        }
        if let Some(ref v) = self.additional_properties {
            map.serialize_entry("additionalProperties", v)?;
        }
        if let Some(ref v) = self.format {
            map.serialize_entry("format", v)?;
        }
        if let Some(ref v) = self.default {
            map.serialize_entry("default", v)?;
        }

        map.end()
    }
}

// Custom Deserialize for JSONSchema to handle "type": "string" OR "type": ["string", "null"]
impl<'de> Deserialize<'de> for JSONSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct JSONSchemaVisitor;

        impl<'de> Visitor<'de> for JSONSchemaVisitor {
            type Value = JSONSchema;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON Schema object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut schema = JSONSchema::default();
                let mut nullable_found = false;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => {
                            // Can be string or array of strings
                            let val: Value = map.next_value()?;
                            if let Some(s) = val.as_str() {
                                // Single type
                                schema.schema_type = Some(
                                    serde_json::from_value(Value::String(s.to_string()))
                                        .map_err(serde::de::Error::custom)?,
                                );
                            } else if let Some(arr) = val.as_array() {
                                // Array type ["string", "null"]
                                let mut has_null = false;
                                let mut primary_type = None;

                                for v in arr {
                                    if let Some(s) = v.as_str() {
                                        if s == "null" {
                                            has_null = true;
                                        } else {
                                            // Assume first non-null type is the primary type
                                            if primary_type.is_none() {
                                                primary_type = Some(
                                                    serde_json::from_value(Value::String(
                                                        s.to_string(),
                                                    ))
                                                    .map_err(serde::de::Error::custom)?,
                                                );
                                            }
                                        }
                                    }
                                }
                                schema.schema_type = primary_type;
                                if has_null {
                                    nullable_found = true;
                                }
                            }
                        }
                        "description" => schema.description = Some(map.next_value()?),
                        "properties" => schema.properties = Some(map.next_value()?),
                        "required" => schema.required = Some(map.next_value()?),
                        "items" => schema.items = Some(map.next_value()?),
                        "enum" => schema.enum_values = Some(map.next_value()?),
                        "additionalProperties" => {
                            schema.additional_properties = Some(map.next_value()?)
                        }
                        "format" => schema.format = Some(map.next_value()?),
                        "default" => schema.default = Some(map.next_value()?),
                        "nullable" => {
                            // Handle explicit nullable field (e.g. from Gemini if parsed via this path)
                            if let Ok(val) = map.next_value::<bool>() {
                                if val {
                                    nullable_found = true;
                                }
                            }
                        }
                        _ => {
                            let _ = map.next_value::<Value>();
                        } // Ignore unknown fields
                    }
                }

                if nullable_found {
                    schema.nullable = Some(true);
                }

                Ok(schema)
            }
        }

        deserializer.deserialize_map(JSONSchemaVisitor)
    }
}

impl JSONSchema {
    /// Create a new Object schema.
    pub fn object() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Object),
            properties: Some(std::collections::BTreeMap::new()),
            required: Some(Vec::new()),
            additional_properties: Some(false),
            ..Default::default()
        }
    }

    /// Create a new String schema.
    pub fn string() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::String),
            ..Default::default()
        }
    }
}

/// Universal Tool Choice Strategy.
/// Unifies OpenAI's ToolChoice and Gemini's ToolConfig.
///
/// 通用工具选择策略。
/// 统一 OpenAI 的 ToolChoice 和 Gemini 的 ToolConfig。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolChoice {
    /// Do not use any tool.
    ///
    /// 不使用任何工具。
    None,

    /// Auto selection.
    ///
    /// 自动选择。
    Auto,

    /// Force use of a tool, but do not specify which one (Gemini `Any` mode).
    ///
    /// 强制使用某个工具，但不指定具体是哪一个 (Gemini `Any` 模式)。
    Required,

    /// Force execution of a specific function.
    ///
    /// 强制调用特定函数。
    Function(String),
}

/// Tool Call Request.
///
/// 工具调用请求。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Tool Execution Result.
///
/// 工具执行结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub name: String,
    pub content: Vec<ContentPart>,
    #[serde(default)]
    pub is_error: bool,
}

impl Tool {
    /// Create a new Function tool.
    ///
    /// 创建一个新的 Function 工具。
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: JSONSchema,
    ) -> Self {
        Self {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
                strict: None,
            },
        }
    }

    /// Set strict mode.
    ///
    /// 设置严格模式。
    pub fn strict(mut self, strict: bool) -> Self {
        self.function.strict = Some(strict);
        self
    }
}

// =========================================================================
//  Converters
// =========================================================================

impl Tool {
    // --- OpenAI Converters ---

    /// Convert to OpenAI Tool.
    ///
    /// 转换为 OpenAI Tool。
    pub fn to_openai(&self) -> OpenAITool {
        let parameters = self
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok());

        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunctionDefinition {
                name: self.function.name.clone(),
                description: self.function.description.clone(),
                parameters,
                strict: self.function.strict,
            },
        }
    }

    // --- Gemini Converters ---

    /// Convert to Gemini FunctionDeclaration.
    /// NOTE: Gemini `Tool` is a container of functions, so this maps to `FunctionDeclaration`.
    ///
    /// 转换为 Gemini FunctionDeclaration。
    /// 注意：Gemini `Tool` 是函数的容器，所以这里映射到 `FunctionDeclaration`。
    pub fn to_gemini_function_declaration(&self) -> GeminiFunctionDeclaration {
        let schema = self
            .function
            .parameters
            .as_ref()
            .and_then(|v| json_schema_to_gemini_schema(v));

        GeminiFunctionDeclaration {
            name: self.function.name.clone(),
            description: self.function.description.clone().unwrap_or_default(),
            parameters: schema,
        }
    }

    // --- Claude Converters ---

    /// Convert to Claude Tool.
    ///
    /// 转换为 Claude Tool。
    pub fn to_claude_tool(&self) -> ClaudeTool {
        let input_schema = self
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok())
            .unwrap_or(Value::Null);

        ClaudeTool::Custom(ClaudeCustomTool {
            name: self.function.name.clone(),
            description: self.function.description.clone(),
            input_schema,
            cache_control: None, // Core doesn't support cache control yet
            r#type: Some("custom".to_string()),
            strict: self.function.strict,
        })
    }
}

/// Convert JSONSchema to Gemini strong-typed Schema.
///
/// 将 JSONSchema 转换为 Gemini 强类型 Schema。
fn json_schema_to_gemini_schema(
    schema: &JSONSchema,
) -> Option<oxide_llm_proto::gemini::v1beta::Schema> {
    use oxide_llm_proto::gemini::v1beta::{Schema as GeminiSchema, Type as GeminiType};

    let schema_type = match schema.schema_type {
        Some(JSONSchemaType::String) => GeminiType::String,
        Some(JSONSchemaType::Number) => GeminiType::Number,
        Some(JSONSchemaType::Integer) => GeminiType::Integer,
        Some(JSONSchemaType::Boolean) => GeminiType::Boolean,
        Some(JSONSchemaType::Array) => GeminiType::Array,
        Some(JSONSchemaType::Object) => GeminiType::Object,
        Some(JSONSchemaType::Null) => GeminiType::TypeUnspecified, // Gemini schema type doesn't support explicit null as primary type
        None => GeminiType::TypeUnspecified,
    };

    let properties = schema.properties.as_ref().map(|props| {
        let mut map = std::collections::HashMap::new();
        for (k, v) in props {
            if let Some(s) = json_schema_to_gemini_schema(v) {
                map.insert(k.clone(), s);
            }
        }
        map
    });

    let items = schema
        .items
        .as_ref()
        .and_then(|v| json_schema_to_gemini_schema(v))
        .map(Box::new);

    Some(GeminiSchema {
        schema_type,
        format: schema.format.clone(),
        description: schema.description.clone(),
        nullable: schema.nullable, // Directly map nullable
        r#enum: schema.enum_values.clone(),
        properties,
        required: schema.required.clone(),
        items,
    })
}

impl ToolChoice {
    // --- OpenAI Converters ---

    pub fn to_openai(&self) -> OpenAIToolChoice {
        match self {
            ToolChoice::None => OpenAIToolChoice::String("none".to_string()),
            ToolChoice::Auto => OpenAIToolChoice::String("auto".to_string()),
            ToolChoice::Required => OpenAIToolChoice::String("required".to_string()),
            ToolChoice::Function(name) => OpenAIToolChoice::Named(OpenAIToolChoiceNamed {
                r#type: "function".to_string(),
                function: OpenAIToolChoiceFunction { name: name.clone() },
            }),
        }
    }

    // --- Gemini Converters ---

    pub fn to_gemini(&self) -> Option<GeminiToolConfig> {
        match self {
            ToolChoice::None => Some(GeminiToolConfig {
                function_calling_config: Some(GeminiFunctionCallingConfig {
                    mode: Some(GeminiFunctionCallingConfigMode::None),
                    allowed_function_names: None,
                }),
                retrieval_config: None,
            }),
            ToolChoice::Auto => Some(GeminiToolConfig {
                function_calling_config: Some(GeminiFunctionCallingConfig {
                    mode: Some(GeminiFunctionCallingConfigMode::Auto),
                    allowed_function_names: None,
                }),
                retrieval_config: None,
            }),
            ToolChoice::Required => Some(GeminiToolConfig {
                function_calling_config: Some(GeminiFunctionCallingConfig {
                    mode: Some(GeminiFunctionCallingConfigMode::Any),
                    allowed_function_names: None,
                }),
                retrieval_config: None,
            }),
            ToolChoice::Function(name) => Some(GeminiToolConfig {
                function_calling_config: Some(GeminiFunctionCallingConfig {
                    mode: Some(GeminiFunctionCallingConfigMode::Any),
                    allowed_function_names: Some(vec![name.clone()]),
                }),
                retrieval_config: None,
            }),
        }
    }

    // --- Claude Converters ---

    pub fn to_claude(&self) -> ClaudeToolChoice {
        match self {
            ToolChoice::None => ClaudeToolChoice::None,
            ToolChoice::Auto => ClaudeToolChoice::Auto {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Required => ClaudeToolChoice::Any {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Function(name) => ClaudeToolChoice::Tool {
                name: name.clone(),
                disable_parallel_tool_use: None,
            },
        }
    }
}

// =========================================================================
//  Reverse Converters (Protocol -> Core)
// =========================================================================

// --- OpenAI -> Core ---

impl TryFrom<OpenAITool> for Tool {
    type Error = String;

    fn try_from(value: OpenAITool) -> Result<Self, Self::Error> {
        if value.r#type != "function" {
            return Err(format!("Unsupported OpenAI tool type: {}", value.r#type));
        }
        let parameters = match value.function.parameters {
            Some(v) => Some(serde_json::from_value(v).map_err(|e| e.to_string())?),
            None => None,
        };
        Ok(Tool {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: value.function.name,
                description: value.function.description,
                parameters,
                strict: value.function.strict,
            },
        })
    }
}

impl From<OpenAIToolChoice> for ToolChoice {
    fn from(value: OpenAIToolChoice) -> Self {
        match value {
            OpenAIToolChoice::String(s) => match s.as_str() {
                "none" => ToolChoice::None,
                "required" => ToolChoice::Required,
                _ => ToolChoice::Auto, // Default to Auto for "auto" or unknowns
            },
            OpenAIToolChoice::Named(named) => ToolChoice::Function(named.function.name),
        }
    }
}

// --- Gemini -> Core ---

impl From<GeminiFunctionDeclaration> for Tool {
    fn from(value: GeminiFunctionDeclaration) -> Self {
        let parameters = value.parameters.map(|s| gemini_schema_to_json_schema(&s));
        Tool {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: value.name,
                description: Some(value.description),
                parameters,
                strict: None, // Gemini doesn't map strict 1:1 on declaration yet
            },
        }
    }
}

impl From<GeminiToolConfig> for ToolChoice {
    fn from(value: GeminiToolConfig) -> Self {
        if let Some(config) = value.function_calling_config {
            match config.mode {
                Some(GeminiFunctionCallingConfigMode::None) => ToolChoice::None,
                Some(GeminiFunctionCallingConfigMode::Auto) => ToolChoice::Auto,
                Some(GeminiFunctionCallingConfigMode::Any) => {
                    if let Some(names) = config.allowed_function_names {
                        if names.len() == 1 {
                            ToolChoice::Function(names[0].clone())
                        } else {
                            // If multiple allowed, we map to Required (Any) as approximation
                            ToolChoice::Required
                        }
                    } else {
                        ToolChoice::Required
                    }
                }
                _ => ToolChoice::Auto,
            }
        } else {
            ToolChoice::Auto
        }
    }
}

// --- Claude -> Core ---

impl TryFrom<ClaudeTool> for Tool {
    type Error = String;

    fn try_from(value: ClaudeTool) -> Result<Self, Self::Error> {
        match value {
            ClaudeTool::Custom(custom) => Ok(Tool {
                r#type: ToolType::Function,
                function: FunctionDefinition {
                    name: custom.name,
                    description: custom.description,
                    parameters: Some(
                        serde_json::from_value(custom.input_schema).map_err(|e| e.to_string())?,
                    ),
                    strict: custom.strict,
                },
            }),
            _ => {
                Err("Only Custom tools are currently supported for generic conversion".to_string())
            }
        }
    }
}

impl From<ClaudeToolChoice> for ToolChoice {
    fn from(value: ClaudeToolChoice) -> Self {
        match value {
            ClaudeToolChoice::None => ToolChoice::None,
            ClaudeToolChoice::Auto { .. } => ToolChoice::Auto,
            ClaudeToolChoice::Any { .. } => ToolChoice::Required,
            ClaudeToolChoice::Tool { name, .. } => ToolChoice::Function(name),
        }
    }
}

// =========================================================================
//  Helpers
// =========================================================================

/// Recursively convert Gemini strong-typed Schema to JSON Value.
///
/// 递归将 Gemini 强类型 Schema 转换为 JSON Value。
/// Recursively convert Gemini strong-typed Schema to JSONSchema.
///
/// 递归将 Gemini 强类型 Schema 转换为 JSONSchema。
fn gemini_schema_to_json_schema(schema: &oxide_llm_proto::gemini::v1beta::Schema) -> JSONSchema {
    use oxide_llm_proto::gemini::v1beta::Type as GeminiType;

    let schema_type = match schema.schema_type {
        GeminiType::String => Some(JSONSchemaType::String),
        GeminiType::Number => Some(JSONSchemaType::Number),
        GeminiType::Integer => Some(JSONSchemaType::Integer),
        GeminiType::Boolean => Some(JSONSchemaType::Boolean),
        GeminiType::Array => Some(JSONSchemaType::Array),
        GeminiType::Object => Some(JSONSchemaType::Object),
        GeminiType::TypeUnspecified => None,
    };

    let properties = schema.properties.as_ref().map(|props| {
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in props {
            map.insert(k.clone(), gemini_schema_to_json_schema(v));
        }
        map
    });

    let items = schema
        .items
        .as_ref()
        .map(|v| Box::new(gemini_schema_to_json_schema(v)));

    JSONSchema {
        schema_type,
        description: schema.description.clone(),
        properties,
        required: schema.required.clone(),
        items,
        enum_values: schema.r#enum.clone(),
        additional_properties: None,
        format: schema.format.clone(),
        default: None,
        nullable: schema.nullable, // Capture nullable from Gemini
    }
}
