use std::future::Future;
use std::pin::Pin;

use crate::message::ContentPart;
use crate::tool::model::Schema;
use crate::tool::model::{FunctionDefinition, ToolDefinition, ToolType};
use serde::{Serialize, de::DeserializeOwned};

/// A type alias for a boxed future returning a tool execution result.
pub type ToolFuture =
    Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, String>> + Send + 'static>>;

/// A trait representing a runnable tool (Type-Erased).
///
/// This is the low-level trait used by the registry.
pub trait ToolRunnable: Send + Sync {
    /// Returns the definition of the tool.
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the provided arguments (JSON Value).
    fn run(&self, args: serde_json::Value) -> ToolFuture;
}

/// A high-level, strongly-typed trait for defining tools with state.
///
/// This trait allows you to define tools using Rust structs for arguments
/// and return types, automatically handling schema generation and serialization.
pub trait Tool: Send + Sync + Clone + 'static {
    /// The name of the tool (must be unique in a registry).
    const NAME: &'static str;

    /// A description of what the tool does.
    const DESCRIPTION: &'static str = "";

    /// The argument type for the tool.
    type Args: DeserializeOwned + Schema + Send;

    /// The output type of the tool.
    type Output: Serialize + Send;

    /// Executes the tool.
    fn run(&self, args: Self::Args) -> impl Future<Output = Result<Self::Output, String>> + Send;
}

/// Blanket implementation of `ToolRunnable` for any type implementing `Tool`.
impl<T: Tool> ToolRunnable for T {
    fn definition(&self) -> ToolDefinition {
        // Generate JSON Schema from the Args type using the Schema trait
        let parameters = T::Args::json_schema();

        ToolDefinition {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: Self::NAME.into(),
                description: if Self::DESCRIPTION.is_empty() {
                    None
                } else {
                    Some(Self::DESCRIPTION.into())
                },
                parameters: Some(parameters),
                strict: None,
            },
        }
    }

    fn run(&self, args: serde_json::Value) -> ToolFuture {
        // Deserialize arguments
        let args_parsed: Result<T::Args, _> = serde_json::from_value(args);

        let tool = self.clone();

        match args_parsed {
            Ok(a) => Box::pin(async move {
                match tool.run(a).await {
                    Ok(output) => {
                        // Serialize output to string for LLM
                        let text = match serde_json::to_string(&output) {
                            Ok(s) => s,
                            Err(e) => {
                                return Err(format!("Tool output serialization error: {}", e));
                            }
                        };
                        Ok(vec![ContentPart::Text {
                            text,
                            signature: None,
                        }])
                    }
                    Err(e) => Err(e),
                }
            }),
            Err(e) => {
                Box::pin(
                    async move { Err(format!("Invalid arguments for tool {}: {}", Self::NAME, e)) },
                )
            }
        }
    }
}
