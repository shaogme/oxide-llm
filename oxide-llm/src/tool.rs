use std::collections::HashMap;
use std::sync::Arc;

use oxide_llm_core::tool::ToolRunnable;
use serde_json::Value;

/// A registry for managing and executing tools.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolRunnable>>,
}

impl ToolRegistry {
    /// Create a new, empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool.
    ///
    /// If a tool with the same name already exists, it will be replaced.
    pub fn register<T>(&mut self, tool: T)
    where
        T: ToolRunnable + 'static,
    {
        let definition = tool.definition();
        self.tools
            .insert(definition.function.name.clone(), Arc::new(tool));
    }

    /// Get all tool definitions.
    pub fn definitions(&self) -> Vec<oxide_llm_core::tool::ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name.
    ///
    /// Returns `None` if the tool is not found.
    pub async fn execute(
        &self,
        name: &str,
        args: Value,
    ) -> Option<Result<Vec<oxide_llm_core::message::ContentPart>, String>> {
        let tool = { self.tools.get(name).cloned() };

        if let Some(tool) = tool {
            Some(tool.run(args).await)
        } else {
            None
        }
    }

    /// Execute a tool by name returning its `ToolFuture`.
    ///
    /// 如果找不到工具则返回 `None`。
    pub fn execute_future(
        &self,
        name: &str,
        args: Value,
    ) -> Option<oxide_llm_core::tool::ToolFuture> {
        let tool = self.tools.get(name).cloned()?;
        Some(tool.run(args))
    }
}
