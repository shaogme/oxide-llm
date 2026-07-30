use crate::tool::{ToolDefinition, ToolGroup, ToolRunnable, ToolSet};
use serde_json::Value;

/// A registry for managing and executing tools using static tool groups.
///
/// 使用静态工具组管理和执行工具的注册表。
#[derive(Debug, Clone, Default)]
pub struct ToolRegistry<G = ()> {
    tools: G,
}

impl ToolRegistry<()> {
    /// Create a new, empty tool registry.
    ///
    /// 创建一个新的空工具注册表。
    pub fn new() -> Self {
        Self { tools: () }
    }
}

impl<G: ToolGroup> ToolRegistry<G> {
    /// Register a tool.
    ///
    /// 向注册表注册一个工具，返回构造了新静态工具链的 ToolRegistry。
    pub fn register<T>(self, tool: T) -> ToolRegistry<ToolSet<G, T>>
    where
        T: ToolRunnable + Clone + 'static,
    {
        ToolRegistry {
            tools: ToolSet {
                head: self.tools,
                tail: tool,
            },
        }
    }

    /// Get all tool definitions.
    ///
    /// 获取所有工具的定义。
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.definitions()
    }

    /// Execute a tool by name.
    ///
    /// Returns `None` if the tool is not found.
    ///
    /// 按名称执行工具。若未找到对应工具则返回 `None`。
    pub fn execute(&self, name: &str, args: Value) -> Option<G::ExecFuture> {
        self.tools.execute(name, args)
    }

    /// Returns a reference to the underlying tool group.
    ///
    /// 返回底层工具组的引用。
    pub fn tools(&self) -> &G {
        &self.tools
    }
}
