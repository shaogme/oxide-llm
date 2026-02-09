use std::future::Future;
use std::pin::Pin;

use crate::message::ContentPart;
use crate::tool::model::Tool;
use serde_json::Value;

/// A type alias for a boxed future returning a tool execution result.
pub type ToolFuture =
    Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, String>> + Send + 'static>>;

/// A trait representing a runnable tool.
///
/// Implementors must provide the tool definition and the execution logic.
pub trait ToolRunnable: Send + Sync {
    /// Returns the definition of the tool (including name, description, and parameters).
    fn definition(&self) -> Tool;

    /// Executes the tool with the provided arguments.
    ///
    /// Returns a future that resolves to the tool's output (as content parts).
    fn run(&self, args: Value) -> ToolFuture;
}
