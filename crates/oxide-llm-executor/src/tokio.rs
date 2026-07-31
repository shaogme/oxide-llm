use oxide_llm_core::{
    message::ContentPart,
    tool::{
        DynTool, Executor, ToolCall, ToolDefinition, ToolExecutionError, ToolRegistry, ToolResult,
        ToolRunnable,
    },
};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tokio::{sync::Semaphore, task::JoinSet};

/// A Tokio-based dynamic tool registry based on `HashMap`.
///
/// 基于 `HashMap` 的 Tokio 动态工具注册表。
#[derive(Default, Clone)]
pub struct TokioToolRegistry {
    tools: Arc<HashMap<String, Arc<dyn DynTool>>>,
}

impl TokioToolRegistry {
    /// Create a new empty `TokioToolRegistry`.
    ///
    /// 创建一个新的空 `TokioToolRegistry`。
    pub fn new() -> Self {
        Self {
            tools: Arc::new(HashMap::new()),
        }
    }

    /// Register a tool into the registry.
    ///
    /// 向注册表注册一个工具。
    pub fn register<T>(&mut self, tool: T)
    where
        T: ToolRunnable + Clone + 'static,
    {
        let def = tool.definition();
        let name = def.function.name.to_string();
        Arc::make_mut(&mut self.tools).insert(name, Arc::new(tool));
    }

    /// Builder-style method to register a tool into the registry.
    ///
    /// 构建器风格的工具注册方法。
    pub fn with_tool<T>(mut self, tool: T) -> Self
    where
        T: ToolRunnable + Clone + 'static,
    {
        self.register(tool);
        self
    }
}

impl ToolRegistry for TokioToolRegistry {
    type ExecFuture =
        Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, ToolExecutionError>> + Send>>;

    fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    fn execute(&self, name: &str, args: serde_json::Value) -> Option<Self::ExecFuture> {
        self.tools.get(name).map(|t| t.execute(args))
    }
}

/// A Tokio-based parallel tool executor with configurable concurrency limit.
///
/// 基于 Tokio 的并行工具执行器，支持可配置的最大并发度。
#[derive(Debug, Clone, Default)]
pub struct TokioExecutor {
    max_concurrency: Option<usize>,
}

impl TokioExecutor {
    /// Create a new `TokioExecutor` with default unlimited concurrency.
    ///
    /// 创建一个新的 `TokioExecutor`，默认不限制并发数。
    pub fn new() -> Self {
        Self {
            max_concurrency: None,
        }
    }

    /// Set the maximum degree of parallelism (concurrency limit).
    ///
    /// 设置最大并发/并行度。为 0 则表示无限制。
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = if max_concurrency == 0 {
            None
        } else {
            Some(max_concurrency)
        };
        self
    }

    /// Get the configured maximum degree of parallelism.
    ///
    /// 获取当前配置的最大并发度。
    pub fn max_concurrency(&self) -> Option<usize> {
        self.max_concurrency
    }
}

impl<R: ToolRegistry> Executor<R> for TokioExecutor {
    type Future<'a>
        = Pin<Box<dyn Future<Output = Result<Vec<ToolResult>, ToolExecutionError>> + Send + 'a>>
    where
        Self: 'a,
        R: 'a;

    fn execute<'a>(&'a self, registry: &'a R, tool_calls: Vec<ToolCall>) -> Self::Future<'a> {
        let max_concurrency = self.max_concurrency;
        Box::pin(async move {
            let semaphore = max_concurrency.map(|c| Arc::new(Semaphore::new(c)));
            let mut join_set = JoinSet::new();

            for (index, tool_call) in tool_calls.into_iter().enumerate() {
                let sem = semaphore.clone();

                if let Some(fut) = registry.execute(&tool_call.name, tool_call.arguments.clone()) {
                    join_set.spawn(async move {
                        let _permit = match sem {
                            Some(s) => s.acquire_owned().await.ok(),
                            None => None,
                        };
                        let res = fut.await;
                        (index, tool_call, Some(res))
                    });
                } else {
                    join_set.spawn(async move { (index, tool_call, None) });
                }
            }

            let mut indexed_results = Vec::new();
            while let Some(res) = join_set.join_next().await {
                let (index, tool_call, exec_res) =
                    res.map_err(|e| ToolExecutionError::Fatal(e.to_string()))?;
                match exec_res {
                    Some(Ok(content)) => {
                        indexed_results.push((
                            index,
                            ToolResult {
                                tool_call_id: tool_call.id,
                                name: tool_call.name,
                                content,
                                is_error: false,
                                signature: tool_call.signature,
                            },
                        ));
                    }
                    Some(Err(ToolExecutionError::Handled(content))) => {
                        indexed_results.push((
                            index,
                            ToolResult {
                                tool_call_id: tool_call.id,
                                name: tool_call.name,
                                content,
                                is_error: true,
                                signature: tool_call.signature,
                            },
                        ));
                    }
                    Some(Err(ToolExecutionError::Fatal(fatal_err))) => {
                        return Err(ToolExecutionError::Fatal(fatal_err));
                    }
                    None => {
                        indexed_results.push((
                            index,
                            ToolResult {
                                tool_call_id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                content: vec![ContentPart::Text {
                                    text: format!("Error: Unknown tool '{}'", tool_call.name),
                                    signature: None,
                                }],
                                is_error: true,
                                signature: tool_call.signature,
                            },
                        ));
                    }
                }
            }

            // Preserve the original tool call order
            indexed_results.sort_by_key(|(idx, _)| *idx);
            Ok(indexed_results.into_iter().map(|(_, res)| res).collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_core::tool::{
        FunctionDefinition, ToolDefinition, ToolError, ToolRunnable, ToolType,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[derive(Clone)]
    struct SlowTool {
        name: String,
        active_counter: Arc<AtomicUsize>,
        max_seen: Arc<AtomicUsize>,
    }

    impl ToolRunnable for SlowTool {
        type Error = String;
        type Future =
            Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, ToolError<String>>> + Send>>;

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                r#type: ToolType::Function,
                function: FunctionDefinition {
                    name: self.name.clone().into(),
                    description: None,
                    parameters: None,
                    strict: None,
                },
            }
        }

        fn run(&self, _args: serde_json::Value) -> Self::Future {
            let active = self.active_counter.clone();
            let max_seen = self.max_seen.clone();
            Box::pin(async move {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                max_seen.fetch_max(current, Ordering::SeqCst);
                sleep(Duration::from_millis(50)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                Ok(vec![ContentPart::Text {
                    text: "done".into(),
                    signature: None,
                }])
            })
        }
    }

    #[tokio::test]
    async fn test_tokio_executor_parallelism_limit() {
        let active_counter = Arc::new(AtomicUsize::new(0));
        let max_seen = Arc::new(AtomicUsize::new(0));

        let tool1 = SlowTool {
            name: "tool1".into(),
            active_counter: active_counter.clone(),
            max_seen: max_seen.clone(),
        };
        let tool2 = SlowTool {
            name: "tool2".into(),
            active_counter: active_counter.clone(),
            max_seen: max_seen.clone(),
        };
        let tool3 = SlowTool {
            name: "tool3".into(),
            active_counter: active_counter.clone(),
            max_seen: max_seen.clone(),
        };

        let registry = TokioToolRegistry::new()
            .with_tool(tool1)
            .with_tool(tool2)
            .with_tool(tool3);

        let executor = TokioExecutor::new().with_max_concurrency(2);
        assert_eq!(executor.max_concurrency(), Some(2));

        let tool_calls = vec![
            ToolCall {
                id: "1".into(),
                name: "tool1".into(),
                arguments: serde_json::json!({}),
                signature: None,
            },
            ToolCall {
                id: "2".into(),
                name: "tool2".into(),
                arguments: serde_json::json!({}),
                signature: None,
            },
            ToolCall {
                id: "3".into(),
                name: "tool3".into(),
                arguments: serde_json::json!({}),
                signature: None,
            },
        ];

        let results = executor.execute(&registry, tool_calls).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].tool_call_id, "1");
        assert_eq!(results[1].tool_call_id, "2");
        assert_eq!(results[2].tool_call_id, "3");
        assert!(max_seen.load(Ordering::SeqCst) <= 2);
    }
}
