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
        #[display("{0}")]
        Custom(E),
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
                    Poll::Ready(Err(err)) => Poll::Ready(Err(ToolError::Custom(err))),
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
        // Deserialize arguments
        let args_parsed: Result<T::Args, _> = serde_json::from_value(args);

        match args_parsed {
            Ok(a) => AutoToolFuture::Executing(self.run(a)),
            Err(e) => {
                let err = ToolError::InvalidArguments(format!(
                    "Invalid arguments for tool {}: {}",
                    Self::NAME,
                    e
                ));
                AutoToolFuture::Failed(Some(err))
            }
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

/// A static branching future that delegates polling to either Left or Right.
///
/// 静态分流 Future，将轮询委托给 Left 或 Right。
pub enum EitherFuture<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Future for EitherFuture<L, R>
where
    L: Future<Output = Result<Vec<ContentPart>, ToolExecutionError>>,
    R: Future<Output = Result<Vec<ContentPart>, ToolExecutionError>>,
{
    type Output = Result<Vec<ContentPart>, ToolExecutionError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match unsafe { self.get_unchecked_mut() } {
            EitherFuture::Left(l) => unsafe { Pin::new_unchecked(l) }.poll(cx),
            EitherFuture::Right(r) => unsafe { Pin::new_unchecked(r) }.poll(cx),
        }
    }
}

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

/// A trait for static tool groups composed of heterogeneous tools.
///
/// 用于由异构工具组成的静态工具组的 Trait。
pub trait ToolGroup: Send + Sync + Clone + 'static {
    /// The combined execution future type of all tools in the group.
    ///
    /// 组内所有工具组合的执行 Future 类型。
    type ExecFuture: Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send;

    /// Returns definitions for all tools in the group.
    ///
    /// 返回组内所有工具的定义。
    fn definitions(&self) -> Vec<ToolDefinition>;

    /// Executes a tool by name if present in the group.
    ///
    /// 若名称匹配组内工具，则执行该工具。
    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture>;
}

// Termination node ()
impl ToolGroup for () {
    type ExecFuture = Ready<Result<Vec<ContentPart>, ToolExecutionError>>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }

    fn execute(&self, _name: &str, _args: serde_json::Value) -> Option<Self::ExecFuture> {
        None
    }
}

/// A recursive static node in a tool group chain.
///
/// 工具组链中的递归静态节点。
#[derive(Clone, Default)]
pub struct ToolSet<Head, Tail> {
    pub head: Head,
    pub tail: Tail,
}

impl<Head, Tail> ToolGroup for ToolSet<Head, Tail>
where
    Head: ToolGroup,
    Tail: ToolRunnable + Clone + 'static,
{
    type ExecFuture = EitherFuture<ToolExecutionFuture<Tail>, Head::ExecFuture>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs = self.head.definitions();
        defs.push(self.tail.definition());
        defs
    }

    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture> {
        let tail_def = self.tail.definition();
        if tail_def.function.name == name {
            Some(EitherFuture::Left(ToolExecutionFuture::new(
                self.tail.clone(),
                args,
            )))
        } else {
            self.head.execute(name, args).map(EitherFuture::Right)
        }
    }
}
