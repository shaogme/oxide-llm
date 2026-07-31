use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    future::{Future, Ready},
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    message::ContentPart,
    tool::model::{FunctionDefinition, Schema, ToolDefinition, ToolType},
};
use diagweave::set;
use serde::{Serialize, de::DeserializeOwned};

set! {
    /// Errors that can occur during tool execution or argument parsing.
    ///
    /// 工具执行或参数解析过程中可能发生的错误。
    pub ToolError<E: Debug + Display + Send + Sync + 'static> = {
        #[display("Invalid arguments: {0}")]
        InvalidArguments(String),
        #[display("Serialization error: {0}")]
        Serialization(String),
        #[display("Execution failure: {0}")]
        Execution(E),
        #[display("Parse failure: {0}")]
        Parse(E),
    }
}

impl<E: Debug + Display + Send + Sync + 'static> From<ToolError<E>> for String {
    fn from(err: ToolError<E>) -> Self {
        err.to_string()
    }
}

/// A trait representing a runnable tool.
///
/// 一个表示可运行工具的 Trait。
pub trait ToolRunnable: Send + Sync {
    /// The error type returned by tool execution.
    ///
    /// 工具执行返回的错误类型。
    type Error: Debug + Display + Send + Sync + 'static;

    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type Future: Future<Output = Result<Vec<ContentPart>, ToolError<Self::Error>>> + Send;

    /// Returns the definition of the tool.
    ///
    /// 返回工具的定义。
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the provided arguments (JSON Value).
    ///
    /// 使用提供的参数（JSON 值）执行工具。
    fn run(&self, args: serde_json::Value) -> Self::Future;

    /// Handles an error occurred during tool execution.
    ///
    /// 处理工具执行过程中产生的错误，返回可传给 LLM 的内容段落或返回致命错误以中断执行。
    fn handle_error(&self, err: ToolError<Self::Error>) -> Result<Vec<ContentPart>, Self::Error> {
        Ok(vec![ContentPart::Text {
            text: format!("Error executing tool: {}", err),
            signature: None,
        }])
    }
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

    /// The error type returned by the tool.
    type Error: Debug + Display + Send + Sync + 'static;

    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type Future: Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// Executes the tool.
    fn run(&self, args: Self::Args) -> Self::Future;

    /// Parses JSON arguments into the strongly-typed `Args` structure.
    ///
    /// 将 JSON 参数解析为强类型的 `Args` 结构体。允许自定义覆写 JSON 解析逻辑。
    fn parse_args(&self, args: serde_json::Value) -> Result<Self::Args, ToolError<Self::Error>> {
        serde_json::from_value(args).map_err(|e| {
            ToolError::InvalidArguments(format!("Invalid arguments for tool {}: {}", Self::NAME, e))
        })
    }

    /// Handles an error occurred during tool execution.
    ///
    /// 处理工具执行过程中产生的错误，返回可传给 LLM 的内容段落或返回致命错误以中断执行。
    fn handle_error(&self, err: ToolError<Self::Error>) -> Result<Vec<ContentPart>, Self::Error> {
        Ok(vec![ContentPart::Text {
            text: format!("Error executing tool: {}", err),
            signature: None,
        }])
    }
}

/// A zero-allocation future returned by the blanket implementation of `ToolRunnable` for `Tool`.
///
/// 由 `Tool` 的 `ToolRunnable` 通用实现返回的零内存分配 Future。
pub enum AutoToolFuture<T: Tool> {
    /// Holds a deferred error during argument parsing or serialization failure.
    Failed(Option<ToolError<T::Error>>),
    /// Holds the inner future of the tool execution.
    Executing(T::Future),
}

impl<T: Tool> Future for AutoToolFuture<T> {
    type Output = Result<Vec<ContentPart>, ToolError<T::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        match this {
            AutoToolFuture::Failed(err_opt) => {
                let err = err_opt
                    .take()
                    .expect("AutoToolFuture polled after completion");
                Poll::Ready(Err(err))
            }
            AutoToolFuture::Executing(fut) => {
                let pinned_fut = unsafe { Pin::new_unchecked(fut) };
                match pinned_fut.poll(cx) {
                    Poll::Ready(Ok(output)) => match serde_json::to_string(&output) {
                        Ok(text) => Poll::Ready(Ok(vec![ContentPart::Text {
                            text,
                            signature: None,
                        }])),
                        Err(e) => {
                            let err = ToolError::Serialization(e.to_string());
                            Poll::Ready(Err(err))
                        }
                    },
                    Poll::Ready(Err(err)) => Poll::Ready(Err(ToolError::Execution(err))),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

/// Blanket implementation of `ToolRunnable` for any type implementing `Tool`.
impl<T: Tool> ToolRunnable for T {
    type Error = T::Error;
    type Future = AutoToolFuture<T>;

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

    fn run(&self, args: serde_json::Value) -> Self::Future {
        // Deserialize arguments using `parse_args`
        let args_parsed = self.parse_args(args);

        match args_parsed {
            Ok(a) => AutoToolFuture::Executing(self.run(a)),
            Err(e) => AutoToolFuture::Failed(Some(e)),
        }
    }

    fn handle_error(&self, err: ToolError<Self::Error>) -> Result<Vec<ContentPart>, Self::Error> {
        T::handle_error(self, err)
    }
}

/// Execution outcome wrapper for tool error handling.
///
/// 工具错误处理的执行结果包装。
#[derive(Debug, Clone, PartialEq)]
pub enum ToolExecutionError {
    /// Non-fatal error formatted into content parts for LLM.
    ///
    /// 格式化为内容段落的非致命错误，供 LLM 继续处理。
    Handled(Vec<ContentPart>),
    /// Fatal error that aborts tool execution and the runner loop.
    ///
    /// 中断工具执行和 Runner 循环的致命错误。
    Fatal(String),
}

impl Display for ToolExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ToolExecutionError::Handled(_) => write!(f, "Tool execution returned handled error"),
            ToolExecutionError::Fatal(err) => write!(f, "Tool execution failed fatally: {}", err),
        }
    }
}

impl Error for ToolExecutionError {}

/// Helper future wrapper that executes a ToolRunnable and maps its error using `handle_error`.
pub struct ToolExecutionFuture<T: ToolRunnable> {
    tool: T,
    future: T::Future,
}

impl<T: ToolRunnable> ToolExecutionFuture<T> {
    pub fn new(tool: T, args: serde_json::Value) -> Self {
        let future = tool.run(args);
        Self { tool, future }
    }
}

impl<T: ToolRunnable> Future for ToolExecutionFuture<T> {
    type Output = Result<Vec<ContentPart>, ToolExecutionError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pinned = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.future) };
        match pinned.poll(cx) {
            Poll::Ready(Ok(content)) => Poll::Ready(Ok(content)),
            Poll::Ready(Err(err)) => match self.tool.handle_error(err) {
                Ok(content) => Poll::Ready(Err(ToolExecutionError::Handled(content))),
                Err(fatal_err) => {
                    Poll::Ready(Err(ToolExecutionError::Fatal(fatal_err.to_string())))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A type-erased trait for tools, enabling dynamic dispatch (`dyn DynTool`).
///
/// 用于工具的类型擦除 Trait，允许动态分发 (`dyn DynTool`)。
pub trait DynTool: Send + Sync {
    /// Returns the definition of the tool.
    ///
    /// 返回工具的定义。
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with JSON arguments, returning a heap-allocated pinned Future.
    ///
    /// 使用 JSON 参数执行工具，返回堆分配的固定 Future。
    fn execute(
        &self,
        args: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send>>;
}

impl<T> DynTool for T
where
    T: ToolRunnable + Clone + 'static,
{
    fn definition(&self) -> ToolDefinition {
        ToolRunnable::definition(self)
    }

    fn execute(
        &self,
        args: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send>> {
        Box::pin(ToolExecutionFuture::new(self.clone(), args))
    }
}

/// A trait for managing and executing tools.
///
/// 用于管理和执行工具的 Trait。
pub trait ToolRegistry: Send + Sync + 'static {
    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type ExecFuture: Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send;

    /// Get all tool definitions.
    ///
    /// 获取所有工具的定义。
    fn definitions(&self) -> Vec<ToolDefinition>;

    /// Execute a tool by name.
    ///
    /// Returns `None` if the tool is not found.
    ///
    /// 按名称执行工具。若未找到对应工具则返回 `None`。
    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture>;
}

// Termination node ()
impl ToolRegistry for () {
    type ExecFuture = Ready<Result<Vec<ContentPart>, ToolExecutionError>>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }

    fn execute(&self, _name: &str, _args: serde_json::Value) -> Option<Self::ExecFuture> {
        None
    }
}
