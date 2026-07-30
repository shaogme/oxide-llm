use crate::message::ContentPart;
use crate::tool::{ToolCall, ToolExecutionError, ToolGroup, ToolResult, registry::ToolRegistry};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A trait for executing tool calls within a `ToolRegistry`.
///
/// 用于在 `ToolRegistry` 中执行工具调用的 Trait。
pub trait Executor<G: ToolGroup>: Send + Sync + 'static {
    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type Future<'a>: Future<Output = Result<Vec<ToolResult>, ToolExecutionError>> + Send + 'a
    where
        Self: 'a,
        G: 'a;

    /// Executes tool calls using the provided tool registry.
    ///
    /// 使用提供的工具注册表执行工具调用。
    fn execute<'a>(
        &'a self,
        registry: &'a ToolRegistry<G>,
        tool_calls: Vec<ToolCall>,
    ) -> Self::Future<'a>;
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

impl<G: ToolGroup> Executor<G> for DefaultExecutor {
    type Future<'a> = ExecuteToolsFuture<G>;

    fn execute<'a>(
        &'a self,
        registry: &'a ToolRegistry<G>,
        tool_calls: Vec<ToolCall>,
    ) -> Self::Future<'a> {
        ExecuteToolsFuture::new(registry.clone(), tool_calls)
    }
}

/// Future that executes a series of tool calls sequentially.
///
/// 顺序执行一系列工具调用的 Future。
pub struct ExecuteToolsFuture<G: ToolGroup> {
    registry: ToolRegistry<G>,
    tool_calls: std::vec::IntoIter<ToolCall>,
    current_exec: Option<(ToolCall, G::ExecFuture)>,
    results: Vec<ToolResult>,
}

impl<G: ToolGroup> ExecuteToolsFuture<G> {
    /// Creates a new `ExecuteToolsFuture`.
    ///
    /// 创建一个新的 `ExecuteToolsFuture`。
    pub fn new(registry: ToolRegistry<G>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            registry,
            tool_calls: tool_calls.into_iter(),
            current_exec: None,
            results: Vec::new(),
        }
    }
}

impl<G: ToolGroup> Unpin for ExecuteToolsFuture<G> {}

impl<G: ToolGroup> Future for ExecuteToolsFuture<G> {
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
