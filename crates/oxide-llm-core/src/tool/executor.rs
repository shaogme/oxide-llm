use crate::{
    message::ContentPart,
    tool::{
        DynTool, ToolCall, ToolDefinition, ToolExecutionError, ToolRegistry, ToolResult,
        ToolRunnable,
    },
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A trait for executing tool calls using a `ToolRegistry`.
///
/// 用于在 `ToolRegistry` 中执行工具调用的 Trait。
pub trait Executor<R: ToolRegistry>: Send + Sync + 'static {
    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type Future<'a>: Future<Output = Result<Vec<ToolResult>, ToolExecutionError>> + Send + 'a
    where
        Self: 'a,
        R: 'a;

    /// Executes tool calls using the provided tool registry.
    ///
    /// 使用提供的工具注册表执行工具调用。
    fn execute<'a>(&'a self, registry: &'a R, tool_calls: Vec<ToolCall>) -> Self::Future<'a>;
}

/// Default sequential tool executor.
///
/// 默认的顺序工具执行器。
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultExecutor;

/// Alias for `DefaultExecutor`.
///
/// `DefaultExecutor` 的别名。
pub type SequentialExecutor = DefaultExecutor;

impl<R: ToolRegistry> Executor<R> for DefaultExecutor {
    type Future<'a> = ExecuteToolsFuture<'a, R>;

    fn execute<'a>(&'a self, registry: &'a R, tool_calls: Vec<ToolCall>) -> Self::Future<'a> {
        ExecuteToolsFuture::new(registry, tool_calls)
    }
}

/// Future that executes a series of tool calls sequentially.
///
/// 顺序执行一系列工具调用的 Future。
pub struct ExecuteToolsFuture<'a, R: ToolRegistry> {
    registry: &'a R,
    tool_calls: std::vec::IntoIter<ToolCall>,
    current_exec: Option<(ToolCall, R::ExecFuture)>,
    results: Vec<ToolResult>,
}

impl<'a, R: ToolRegistry> ExecuteToolsFuture<'a, R> {
    /// Creates a new `ExecuteToolsFuture`.
    ///
    /// 创建一个新的 `ExecuteToolsFuture`。
    pub fn new(registry: &'a R, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            registry,
            tool_calls: tool_calls.into_iter(),
            current_exec: None,
            results: Vec::new(),
        }
    }
}

impl<'a, R: ToolRegistry> Unpin for ExecuteToolsFuture<'a, R> {}

impl<'a, R: ToolRegistry> Future for ExecuteToolsFuture<'a, R> {
    type Output = Result<Vec<ToolResult>, ToolExecutionError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        loop {
            if let Some((_, ref mut fut)) = this.current_exec {
                let pinned_fut = unsafe { Pin::new_unchecked(fut) };
                match pinned_fut.poll(cx) {
                    Poll::Ready(res) => {
                        let (tool_call, _) = this.current_exec.take().unwrap();
                        match res {
                            Ok(content) => {
                                this.results.push(ToolResult {
                                    tool_call_id: tool_call.id.clone(),
                                    name: tool_call.name.clone(),
                                    content,
                                    is_error: false,
                                    signature: tool_call.signature.clone(),
                                });
                            }
                            Err(ToolExecutionError::Handled(content)) => {
                                this.results.push(ToolResult {
                                    tool_call_id: tool_call.id.clone(),
                                    name: tool_call.name.clone(),
                                    content,
                                    is_error: true,
                                    signature: tool_call.signature.clone(),
                                });
                            }
                            Err(ToolExecutionError::Fatal(fatal_err)) => {
                                return Poll::Ready(Err(ToolExecutionError::Fatal(fatal_err)));
                            }
                        }
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            if let Some(tool_call) = this.tool_calls.next() {
                let name = tool_call.name.clone();
                let args = tool_call.arguments.clone();
                if let Some(fut) = this.registry.execute(&name, args) {
                    this.current_exec = Some((tool_call, fut));
                } else {
                    let result = ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        content: vec![ContentPart::Text {
                            text: format!("Error: Unknown tool '{}'", tool_call.name),
                            signature: None,
                        }],
                        is_error: true,
                        signature: tool_call.signature.clone(),
                    };
                    this.results.push(result);
                }
            } else {
                return Poll::Ready(Ok(std::mem::take(&mut this.results)));
            }
        }
    }
}

/// A general-purpose tool registry backed by type-erased [`DynTool`] objects.
///
/// This allows mixing tools of different types in a single registry without
/// needing hand-written `EitherFuture` branches.
///
/// 基于类型擦除的通用工具注册表，支持将不同类型的工具混合注册而无需手写类型分支。
pub struct DynToolRegistry {
    tools: Vec<Box<dyn DynTool>>,
}

impl Default for DynToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DynToolRegistry {
    /// Creates a new empty `DynToolRegistry`.
    ///
    /// 创建一个空的 `DynToolRegistry`。
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Registers a tool into the registry.
    ///
    /// 将一个工具注册到注册表中。
    pub fn register<T>(&mut self, tool: T) -> &mut Self
    where
        T: ToolRunnable + Clone + 'static,
    {
        self.tools.push(Box::new(tool));
        self
    }

    /// Builder-style registration: consumes `self`, registers the tool, and returns `self`.
    ///
    /// Builder 风格的注册：消耗 `self`，注册工具后返回 `self`。
    pub fn with<T>(mut self, tool: T) -> Self
    where
        T: ToolRunnable + Clone + 'static,
    {
        self.register(tool);
        self
    }
}

impl ToolRegistry for DynToolRegistry {
    type ExecFuture =
        Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send>>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture> {
        self.tools
            .iter()
            .find(|t| t.definition().function.name == name)
            .map(|t| t.execute(args))
    }
}
