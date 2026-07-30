use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Definition of a function that the model may call.
///
/// 模型可调用的函数定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function to be called.
    ///
    /// 要调用的函数名称。
    pub name: StaticRefStr,

    /// A description of what the function does, used by the model to choose when and how to call the function.
    ///
    /// 函数的功能描述，模型用于判断何时以及如何调用该函数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,

    /// The parameters the function accepts, described as a JSON Schema object.
    ///
    /// 函数接收的参数，以 JSON Schema 对象形式描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,

    /// Whether to enable strict schema adherence when generating function calls.
    ///
    /// 是否开启严格模式以在生成函数调用时遵守 Schema 约束。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Function specification for forcing a specific tool choice.
///
/// 强制选择特定工具时的函数规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    /// The name of the function to call.
    ///
    /// 要调用的函数名称。
    pub name: StaticRefStr,
}

/// Tool definition for OpenAI Response API.
///
/// OpenAI Response API 中的工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The type of the tool.
    ///
    /// 工具的类型。
    pub r#type: String,

    /// The name of the tool.
    ///
    /// 工具的名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A description of what the tool does.
    ///
    /// 工具的功能描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The parameters the tool accepts, described as a JSON Schema object.
    ///
    /// 工具接收的参数，以 JSON Schema 对象形式描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,

    /// The function definition if the tool is a custom function.
    ///
    /// 若工具为自定义函数时的函数定义。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionDefinition>,

    /// Whether to enable strict schema adherence for this tool.
    ///
    /// 是否对此工具启用严格 Schema 约束。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Controls which tool (if any) is called by the model.
///
/// 控制模型要调用的工具（如果有）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Specify mode directly, e.g., "auto", "none", "required".
    ///
    /// 直接指定模式字符串，例如 "auto", "none", "required"。
    Mode(String),

    /// Force the model to call a specific tool or function.
    ///
    /// 强制模型调用指定的工具或函数。
    Function {
        /// The type of the tool (e.g., "function").
        ///
        /// 工具类型（例如 "function"）。
        r#type: String,

        /// The name of the custom tool.
        ///
        /// 自定义工具名称。
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,

        /// Function details if specifying a function call.
        ///
        /// 若指定函数调用时的函数详细信息。
        #[serde(skip_serializing_if = "Option::is_none")]
        function: Option<ToolChoiceFunction>,
    },
}
