use serde_json::Value;

use super::model::{
    FunctionDefinition, JSONSchema, JSONSchemaType, ToolChoice, ToolDefinition, ToolType,
};
use oxide_llm_proto::{
    claude::v1::messages::request::{
        CustomTool as ClaudeCustomTool, Tool as ClaudeTool, ToolChoice as ClaudeToolChoice,
    },
    gemini::v1beta::generate_content::{
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

// =========================================================================
//  Tool Adapter Traits
// =========================================================================

/// Extension trait for `ToolDefinition` to support protocol conversion.
///
/// `ToolDefinition` 的扩展特性，支持协议转换。
pub trait ToolAdapter {
    /// Convert to OpenAI Tool.
    ///
    /// 转换为 OpenAI Tool。
    fn to_openai(&self) -> OpenAITool;

    /// Convert to Gemini FunctionDeclaration.
    ///
    /// 转换为 Gemini FunctionDeclaration。
    fn to_gemini_function_declaration(&self) -> GeminiFunctionDeclaration;

    /// Convert to Claude Tool.
    ///
    /// 转换为 Claude Tool。
    fn to_claude_tool(&self) -> ClaudeTool;
}

/// Extension trait for `ToolChoice` to support protocol conversion.
///
/// `ToolChoice` 的扩展特性，支持协议转换。
pub trait ToolChoiceAdapter {
    fn to_openai(&self) -> OpenAIToolChoice;
    fn to_gemini(&self) -> Option<GeminiToolConfig>;
    fn to_claude(&self) -> ClaudeToolChoice;
}

// =========================================================================
//  Tool Adapter Implementations
// =========================================================================

impl ToolAdapter for ToolDefinition {
    fn to_openai(&self) -> OpenAITool {
        let parameters = self
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok());

        OpenAITool {
            r#type: "function".into(),
            function: OpenAIFunctionDefinition {
                name: self.function.name.clone(),
                description: self.function.description.clone(),
                parameters,
                strict: self.function.strict,
            },
        }
    }

    fn to_gemini_function_declaration(&self) -> GeminiFunctionDeclaration {
        let schema = self
            .function
            .parameters
            .as_ref()
            .and_then(json_schema_to_gemini_schema);

        GeminiFunctionDeclaration {
            name: self.function.name.clone(),
            description: self.function.description.clone().unwrap_or_default(),
            parameters: schema,
        }
    }

    fn to_claude_tool(&self) -> ClaudeTool {
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
            r#type: Some("custom".into()),
            strict: self.function.strict,
        })
    }
}

impl ToolChoiceAdapter for ToolChoice {
    fn to_openai(&self) -> OpenAIToolChoice {
        match self {
            ToolChoice::None => OpenAIToolChoice::String("none".into()),
            ToolChoice::Auto => OpenAIToolChoice::String("auto".into()),
            ToolChoice::Required => OpenAIToolChoice::String("required".into()),
            ToolChoice::Function { name } => OpenAIToolChoice::Named(OpenAIToolChoiceNamed {
                r#type: "function".into(),
                function: OpenAIToolChoiceFunction { name: name.clone() },
            }),
        }
    }

    fn to_gemini(&self) -> Option<GeminiToolConfig> {
        let (mode, allowed_function_names) = match self {
            ToolChoice::None => (Some(GeminiFunctionCallingConfigMode::None), None),
            ToolChoice::Auto => (Some(GeminiFunctionCallingConfigMode::Auto), None),
            ToolChoice::Required => (Some(GeminiFunctionCallingConfigMode::Any), None),
            ToolChoice::Function { name } => (
                Some(GeminiFunctionCallingConfigMode::Any),
                Some(vec![name.clone()]),
            ),
        };

        Some(GeminiToolConfig {
            function_calling_config: Some(GeminiFunctionCallingConfig {
                mode,
                allowed_function_names,
            }),
            retrieval_config: None,
        })
    }

    fn to_claude(&self) -> ClaudeToolChoice {
        match self {
            ToolChoice::None => ClaudeToolChoice::None,
            ToolChoice::Auto => ClaudeToolChoice::Auto {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Required => ClaudeToolChoice::Any {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Function { name } => ClaudeToolChoice::Tool {
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

impl TryFrom<OpenAITool> for ToolDefinition {
    type Error = String;

    fn try_from(value: OpenAITool) -> Result<Self, Self::Error> {
        if value.r#type != "function" {
            return Err(format!("Unsupported OpenAI tool type: {}", value.r#type));
        }
        let parameters = match value.function.parameters {
            Some(v) => Some(serde_json::from_value(v).map_err(|e| e.to_string())?),
            None => None,
        };
        Ok(ToolDefinition {
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
            OpenAIToolChoice::String(s) => match s.as_ref() {
                "none" => ToolChoice::None,
                "required" => ToolChoice::Required,
                _ => ToolChoice::Auto, // Default to Auto for "auto" or unknowns
            },
            OpenAIToolChoice::Named(named) => ToolChoice::Function {
                name: named.function.name,
            },
        }
    }
}

// --- Gemini -> Core ---

impl From<GeminiFunctionDeclaration> for ToolDefinition {
    fn from(value: GeminiFunctionDeclaration) -> Self {
        let parameters = value.parameters.map(|s| gemini_schema_to_json_schema(&s));
        ToolDefinition {
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
        let Some(config) = value.function_calling_config else {
            return ToolChoice::Auto;
        };

        match config.mode {
            Some(GeminiFunctionCallingConfigMode::None) => ToolChoice::None,
            Some(GeminiFunctionCallingConfigMode::Auto) => ToolChoice::Auto,
            Some(GeminiFunctionCallingConfigMode::Any) => config
                .allowed_function_names
                .filter(|names| names.len() == 1)
                .map(|names| ToolChoice::Function {
                    name: names[0].clone(),
                })
                .unwrap_or(ToolChoice::Required),
            _ => ToolChoice::Auto,
        }
    }
}

// --- Claude -> Core ---

impl TryFrom<ClaudeTool> for ToolDefinition {
    type Error = String;

    fn try_from(value: ClaudeTool) -> Result<Self, Self::Error> {
        match value {
            ClaudeTool::Custom(custom) => Ok(ToolDefinition {
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
            ClaudeToolChoice::Tool { name, .. } => ToolChoice::Function { name },
        }
    }
}

// =========================================================================
//  Helpers
// =========================================================================

/// Convert JSONSchema to Gemini strong-typed Schema.
///
/// 将 JSONSchema 转换为 Gemini 强类型 Schema。
pub fn json_schema_to_gemini_schema(
    schema: &JSONSchema,
) -> Option<oxide_llm_proto::gemini::v1beta::generate_content::Schema> {
    use oxide_llm_proto::gemini::v1beta::generate_content::{
        Schema as GeminiSchema, Type as GeminiType,
    };

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

/// Recursively convert Gemini strong-typed Schema to JSONSchema.
///
/// 递归将 Gemini 强类型 Schema 转换为 JSONSchema。
pub fn gemini_schema_to_json_schema(
    schema: &oxide_llm_proto::gemini::v1beta::generate_content::Schema,
) -> JSONSchema {
    use oxide_llm_proto::gemini::v1beta::generate_content::Type as GeminiType;

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
