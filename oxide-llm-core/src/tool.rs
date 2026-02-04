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
    pub parameters: Option<Value>,

    /// Strict mode (OpenAI specific, but good to preserve).
    ///
    /// 严格模式 (OpenAI 特有，但在核心层保留有助于兼容)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
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
        parameters: Value,
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
        OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunctionDefinition {
                name: self.function.name.clone(),
                description: self.function.description.clone(),
                parameters: self.function.parameters.clone(),
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
            .and_then(|v| json_value_to_gemini_schema(v));

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
        ClaudeTool::Custom(ClaudeCustomTool {
            name: self.function.name.clone(),
            description: self.function.description.clone(),
            input_schema: self.function.parameters.clone().unwrap_or(Value::Null),
            cache_control: None, // Core doesn't support cache control yet
            typ: Some("custom".to_string()),
            strict: self.function.strict,
        })
    }
}

/// Recursively convert generic JSON Schema (Value) to Gemini strong-typed Schema.
///
/// 递归将通用 JSON Schema (Value) 转换为 Gemini 强类型 Schema。
fn json_value_to_gemini_schema(v: &Value) -> Option<oxide_llm_proto::gemini::v1beta::Schema> {
    use oxide_llm_proto::gemini::v1beta::{Schema as GeminiSchema, Type as GeminiType};

    let obj = v.as_object()?;

    // 1. Map 'type' field (JSON schema lowercase -> Gemini CONSTANT_CASE)
    // Handle both string "type" and array ["type", "null"]
    let (schema_type, is_nullable_from_type) = match obj.get("type") {
        Some(Value::String(s)) => (map_type_str(s), false),
        Some(Value::Array(arr)) => {
            let has_null = arr.iter().any(|v| v.as_str() == Some("null"));
            let primary_type = arr
                .iter()
                .filter_map(|v| v.as_str())
                .find(|&s| s != "null")
                .map(|s| map_type_str(s))
                .unwrap_or(GeminiType::TypeUnspecified);
            (primary_type, has_null)
        }
        _ => (GeminiType::TypeUnspecified, false),
    };

    // 2. Recursively handle 'properties'
    let properties = obj
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|props| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in props {
                if let Some(s) = json_value_to_gemini_schema(v) {
                    map.insert(k.clone(), s);
                }
            }
            map
        });

    // 3. Recursively handle 'items' (for arrays)
    let items = obj
        .get("items")
        .and_then(|v| json_value_to_gemini_schema(v))
        .map(Box::new);

    // 4. Handle 'required'
    let required = obj.get("required").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect()
    });

    // 5. Handle 'enum'
    let enum_vals = obj.get("enum").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect()
    });

    // Determine final nullable state: explicit 'nullable' field OR array type
    let nullable = is_nullable_from_type
        || obj
            .get("nullable")
            .and_then(|b| b.as_bool())
            .unwrap_or(false);

    Some(GeminiSchema {
        schema_type,
        format: obj.get("format").and_then(|s| s.as_str()).map(String::from),
        description: obj
            .get("description")
            .and_then(|s| s.as_str())
            .map(String::from),
        nullable: if nullable { Some(true) } else { None },
        r#enum: enum_vals,
        properties,
        required,
        items,
    })
}

fn map_type_str(s: &str) -> oxide_llm_proto::gemini::v1beta::Type {
    use oxide_llm_proto::gemini::v1beta::Type as GeminiType;
    match s {
        "string" => GeminiType::String,
        "number" => GeminiType::Number,
        "integer" => GeminiType::Integer,
        "boolean" => GeminiType::Boolean,
        "array" => GeminiType::Array,
        "object" => GeminiType::Object,
        _ => GeminiType::TypeUnspecified,
    }
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
        Ok(Tool {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: value.function.name,
                description: value.function.description,
                parameters: value.function.parameters,
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
        let parameters = value.parameters.map(|s| gemini_schema_to_json_value(&s));
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
                    parameters: Some(custom.input_schema),
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
fn gemini_schema_to_json_value(schema: &oxide_llm_proto::gemini::v1beta::Schema) -> Value {
    use oxide_llm_proto::gemini::v1beta::Type as GeminiType;
    let mut map = serde_json::Map::new();

    // Type
    let type_str = match schema.schema_type {
        GeminiType::String => "string",
        GeminiType::Number => "number",
        GeminiType::Integer => "integer",
        GeminiType::Boolean => "boolean",
        GeminiType::Array => "array",
        GeminiType::Object => "object",
        _ => "string", // Default or ignore
    };
    map.insert("type".to_string(), Value::String(type_str.to_string()));

    // Format
    if let Some(ref f) = schema.format {
        map.insert("format".to_string(), Value::String(f.clone()));
    }
    // Description
    if let Some(ref d) = schema.description {
        map.insert("description".to_string(), Value::String(d.clone()));
    }
    // Nullable
    if let Some(n) = schema.nullable {
        if n {
            // In JSON schema nullable is usually a type list ["string", "null"] or just handled by logic
            // But simple simple field 'nullable' is OpenAPI specific.
            map.insert("nullable".to_string(), Value::Bool(true));
        }
    }

    // Enum
    if let Some(ref e) = schema.r#enum {
        map.insert(
            "enum".to_string(),
            Value::Array(e.iter().map(|s| Value::String(s.clone())).collect()),
        );
    }

    // Properties
    if let Some(ref props) = schema.properties {
        let mut props_map = serde_json::Map::new();
        for (k, v) in props {
            props_map.insert(k.clone(), gemini_schema_to_json_value(v));
        }
        map.insert("properties".to_string(), Value::Object(props_map));
    }

    // Items
    if let Some(ref items) = schema.items {
        map.insert("items".to_string(), gemini_schema_to_json_value(items));
    }

    // Required
    if let Some(ref req) = schema.required {
        map.insert(
            "required".to_string(),
            Value::Array(req.iter().map(|s| Value::String(s.clone())).collect()),
        );
    }

    Value::Object(map)
}
