use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::ContentPart;

/// Universal Tool Definition.
///
/// Designed to cover OpenAI `Tool`, Gemini `FunctionDeclaration` (inside Tool), and Claude `Tool`.
///
/// 通用工具定义。
/// 旨在覆盖 OpenAI `Tool`，Gemini `FunctionDeclaration` (在 Tool 内部)，以及 Claude `Tool`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
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
// Helper for parsing type field which can be "string" or ["string", "null"]
fn parse_complex_type(val: Value) -> Result<(Option<JSONSchemaType>, bool), String> {
    match val {
        Value::String(s) => {
            let t: JSONSchemaType =
                serde_json::from_value(Value::String(s)).map_err(|e| e.to_string())?;
            Ok((Some(t), false))
        }
        Value::Array(arr) => {
            let mut has_null = false;
            let mut primary_type = None;

            for v in arr {
                if let Some(s) = v.as_str() {
                    if s == "null" {
                        has_null = true;
                    } else if primary_type.is_none() {
                        let t: JSONSchemaType =
                            serde_json::from_value(Value::String(s.to_string()))
                                .map_err(|e| e.to_string())?;
                        primary_type = Some(t);
                    }
                }
            }
            Ok((primary_type, has_null))
        }
        _ => Ok((None, false)),
    }
}

struct JSONSchemaVisitor;

impl<'de> serde::de::Visitor<'de> for JSONSchemaVisitor {
    type Value = JSONSchema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON Schema object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut schema = JSONSchema::default();
        let mut nullable_found = false;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    let val: Value = map.next_value()?;
                    let (t, n) = parse_complex_type(val).map_err(serde::de::Error::custom)?;
                    schema.schema_type = t;
                    if n {
                        nullable_found = true;
                    }
                }
                "description" => schema.description = Some(map.next_value()?),
                "properties" => schema.properties = Some(map.next_value()?),
                "required" => schema.required = Some(map.next_value()?),
                "items" => schema.items = Some(map.next_value()?),
                "enum" => schema.enum_values = Some(map.next_value()?),
                "additionalProperties" => schema.additional_properties = Some(map.next_value()?),
                "format" => schema.format = Some(map.next_value()?),
                "default" => schema.default = Some(map.next_value()?),
                "nullable" => {
                    let val: Value = map.next_value()?;
                    if val.as_bool() == Some(true) {
                        nullable_found = true;
                    }
                }
                _ => {
                    let _ = map.next_value::<Value>();
                }
            }
        }

        if nullable_found {
            schema.nullable = Some(true);
        }

        Ok(schema)
    }
}

impl<'de> Deserialize<'de> for JSONSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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

    /// Create a new Number schema.
    pub fn number() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Number),
            ..Default::default()
        }
    }

    /// Create a new Integer schema.
    pub fn integer() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Integer),
            ..Default::default()
        }
    }

    /// Create a new Boolean schema.
    pub fn boolean() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Boolean),
            ..Default::default()
        }
    }

    /// Create a new Array schema.
    pub fn array(items: JSONSchema) -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Array),
            items: Some(Box::new(items)),
            ..Default::default()
        }
    }

    /// Create a new Null schema.
    pub fn null() -> Self {
        Self {
            schema_type: Some(JSONSchemaType::Null),
            ..Default::default()
        }
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set enum values (for string types mostly).
    pub fn enum_values(mut self, values: Vec<impl Into<String>>) -> Self {
        self.enum_values = Some(values.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Add a property to the object.
    pub fn property(mut self, name: impl Into<String>, schema: JSONSchema) -> Self {
        if let Some(props) = &mut self.properties {
            props.insert(name.into(), schema);
        }
        self
    }

    /// Add a required property to the object.
    pub fn required_property(mut self, name: impl Into<String>, schema: JSONSchema) -> Self {
        let name = name.into();
        if let Some(props) = &mut self.properties {
            props.insert(name.clone(), schema);
        }
        if let Some(req) = &mut self.required {
            req.push(name);
        }
        self
    }
}

/// Builder for creating a `Tool`.
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    parameters: Option<JSONSchema>,
    strict: Option<bool>,
}

impl ToolBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: None,
            strict: None,
        }
    }

    /// Set tool description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set tool parameters (input schema).
    pub fn parameters(mut self, schema: JSONSchema) -> Self {
        self.parameters = Some(schema);
        self
    }

    /// Set strict mode.
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Build the `Tool`.
    pub fn build(self) -> ToolDefinition {
        ToolDefinition {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: self.name,
                description: self.description,
                parameters: self.parameters,
                strict: self.strict,
            },
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
    Function { name: String },
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

impl ToolDefinition {
    /// Create a builder for defining a Tool.
    pub fn builder(name: impl Into<String>) -> ToolBuilder {
        ToolBuilder::new(name)
    }

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

/// A trait for generating JSON Schema from Rust types.
///
/// 用于从 Rust 类型生成 JSON Schema。
pub trait Schema {
    fn json_schema() -> JSONSchema;

    /// Is this field optional?
    ///
    /// 该字段是否可选？
    fn is_optional() -> bool {
        false
    }
}

impl Schema for String {
    fn json_schema() -> JSONSchema {
        JSONSchema::string()
    }
}

impl Schema for bool {
    fn json_schema() -> JSONSchema {
        JSONSchema::boolean()
    }
}

macro_rules! impl_schema_number {
    ($($t:ty),*) => {
        $(
            impl Schema for $t {
                fn json_schema() -> JSONSchema {
                    JSONSchema::number()
                }
            }
        )*
    };
}

impl_schema_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

impl<T: Schema> Schema for Vec<T> {
    fn json_schema() -> JSONSchema {
        JSONSchema::array(T::json_schema())
    }
}

impl<T: Schema> Schema for Option<T> {
    fn json_schema() -> JSONSchema {
        let mut schema = T::json_schema();
        schema.nullable = Some(true);
        schema
    }

    fn is_optional() -> bool {
        true
    }
}

impl<T: Schema> Schema for Box<T> {
    fn json_schema() -> JSONSchema {
        T::json_schema()
    }

    fn is_optional() -> bool {
        T::is_optional()
    }
}

/// Macro to easily implement Schema for structs.
///
/// 用于便捷实现 Schema 的宏。
///
/// Usage:
/// ```rust
/// use oxide_llm_core::impl_schema;
///
/// struct MyStruct {
///     name: String,
///     age: i32,
/// }
///
/// impl_schema!(
///     MyStruct,
///     "My Struct Description",
///     {
///         name: String => "Name of the person",
///         age: i32 => "Age of the person"
///     }
/// );
/// ```
#[macro_export]
macro_rules! impl_schema {
    ($struct_name:ident, $description:expr, {
        $( $field:ident : $type:ty => $desc:expr ),* $(,)?
    }) => {
        impl $crate::tool::model::Schema for $struct_name {
            fn json_schema() -> $crate::tool::model::JSONSchema {
                #[allow(unused_mut)]
                let mut schema = $crate::tool::model::JSONSchema::object()
                    .description($description);

                $(
                {
                    let field_schema = <$type as $crate::tool::model::Schema>::json_schema()
                        .description($desc);

                    if !<$type as $crate::tool::model::Schema>::is_optional() {
                        schema = schema.required_property(stringify!($field), field_schema);
                    } else {
                        schema = schema.property(stringify!($field), field_schema);
                    }
                }
                )*

                schema
            }
        }
    };
}
