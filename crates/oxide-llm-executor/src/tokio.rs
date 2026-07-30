use oxide_llm_core::{
    message::ContentPart,
    tool::{Executor, ToolCall, ToolExecutionError, ToolGroup, ToolRegistry, ToolResult},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

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

impl<G: ToolGroup> Executor<G> for TokioExecutor {
    type Future<'a>
        = Pin<Box<dyn Future<Output = Result<Vec<ToolResult>, ToolExecutionError>> + Send + 'a>>
    where
        Self: 'a,
        G: 'a;

    fn execute<'a>(
        &'a self,
        registry: &'a ToolRegistry<G>,
        tool_calls: Vec<ToolCall>,
    ) -> Self::Future<'a> {
        let max_concurrency = self.max_concurrency;
        Box::pin(async move {
            let semaphore = max_concurrency.map(|c| Arc::new(Semaphore::new(c)));
            let mut join_set = JoinSet::new();

            for (index, tool_call) in tool_calls.into_iter().enumerate() {
                let registry = registry.clone();
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
    use oxide_llm_core::tool::{FunctionDefinition, ToolDefinition, ToolRunnable, ToolType};
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
        type Future = Pin<Box<dyn Future<Output = Result<Vec<ContentPart>, String>> + Send>>;

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

        let registry = ToolRegistry::new()
            .register(tool1)
            .register(tool2)
            .register(tool3);

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
