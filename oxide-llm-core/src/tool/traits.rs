use std::future::{Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::message::ContentPart;
use crate::tool::model::{FunctionDefinition, Schema, ToolDefinition, ToolType};
use serde::{Serialize, de::DeserializeOwned};

/// A trait representing a runnable tool.
///
/// 一个表示可运行工具的 Trait。
pub trait ToolRunnable: Send + Sync {
    /// The future type returned by tool execution.
    ///
    /// 工具执行返回的 Future 类型。
    type Future: Future<Output = Result<Vec<ContentPart>, String>> + Send;

    /// Returns the definition of the tool.
    ///
    /// 返回工具的定义。
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the provided arguments (JSON Value).
    ///
    /// 使用提供的参数（JSON 值）执行工具。
    fn run(&self, args: serde_json::Value) -> Self::Future;
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
    type Future = Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, String>> + Send>>;

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

/// A static branching future that delegates polling to either Left or Right.
///
/// 静态分流 Future，将轮询委托给 Left 或 Right。
pub enum EitherFuture<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Future for EitherFuture<L, R>
where
    L: Future<Output = Result<Vec<ContentPart>, String>>,
    R: Future<Output = Result<Vec<ContentPart>, String>>,
{
    type Output = Result<Vec<ContentPart>, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match unsafe { self.get_unchecked_mut() } {
            EitherFuture::Left(l) => unsafe { Pin::new_unchecked(l) }.poll(cx),
            EitherFuture::Right(r) => unsafe { Pin::new_unchecked(r) }.poll(cx),
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
    type ExecFuture: Future<Output = Result<Vec<ContentPart>, String>> + Send;

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
    type ExecFuture = Ready<Result<Vec<ContentPart>, String>>;

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
    type ExecFuture = EitherFuture<Tail::Future, Head::ExecFuture>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs = self.head.definitions();
        defs.push(self.tail.definition());
        defs
    }

    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture> {
        let tail_def = self.tail.definition();
        if tail_def.function.name == name {
            Some(EitherFuture::Left(self.tail.run(args)))
        } else {
            self.head.execute(name, args).map(EitherFuture::Right)
        }
    }
}
