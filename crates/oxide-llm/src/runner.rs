use crate::ChatAgent;
use crate::core::message::{ChatStream, ChatStreamEvent, ContentPart, Message, Role};
use crate::core::state::ConversationState;
pub use crate::core::tool::{
    DefaultExecutor, Executor, SequentialExecutor, ToolCall, ToolRegistry, ToolRunnable,
};
use crate::error::AgentError;
use futures::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// High-level runner for orchestrating agent interactions and tool executions.
///
/// 用于协调 Agent 交互和工具执行的高级 Runner。
#[derive(Clone, Default)]
pub struct Runner<A, R = (), E = DefaultExecutor> {
    agent: A,
    registry: R,
    executor: E,
    max_turns: usize,
    auto_sync_tools: bool,
}

impl<A> Runner<A, (), DefaultExecutor> {
    /// Creates a new `Runner` wrapping the given agent with default max turns (5).
    ///
    /// 创建一个新的 `Runner` 包装指定的 Agent，默认最大轮次数为 5。
    pub fn new(agent: A) -> Self {
        Self {
            agent,
            registry: (),
            executor: DefaultExecutor,
            max_turns: 5,
            auto_sync_tools: false,
        }
    }
}

impl<A, R: ToolRegistry, E: Executor<R>> Runner<A, R, E> {
    /// Sets the maximum number of turns for interaction loops.
    ///
    /// 设置交互循环的最大轮次数。
    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Sets whether to automatically synchronize registered tools into `ConversationState` during `run_stream`.
    ///
    /// 设置是否在 `run_stream` 时自动将已注册的工具同步到 `ConversationState` 中。
    pub fn with_auto_sync_tools(mut self, auto_sync: bool) -> Self {
        self.auto_sync_tools = auto_sync;
        self
    }

    /// Sets the custom executor for tool executions.
    ///
    /// 设置自定义的工具执行器。
    pub fn with_executor<E2: Executor<R>>(self, executor: E2) -> Runner<A, R, E2> {
        Runner {
            agent: self.agent,
            registry: self.registry,
            executor,
            max_turns: self.max_turns,
            auto_sync_tools: self.auto_sync_tools,
        }
    }

    /// Sets the tool registry for the runner.
    ///
    /// 设置 Runner 的工具注册表。
    pub fn with_registry<R2: ToolRegistry>(self, registry: R2) -> Runner<A, R2, E>
    where
        E: Executor<R2>,
    {
        Runner {
            agent: self.agent,
            registry,
            executor: self.executor,
            max_turns: self.max_turns,
            auto_sync_tools: self.auto_sync_tools,
        }
    }

    /// Synchronizes registered tools into the conversation state.
    ///
    /// 将已注册的工具同步填充到对话状态中。
    pub fn sync_tools(&self, state: &mut ConversationState) {
        let definitions = self.registry.definitions();
        for def in definitions {
            if !state
                .tools()
                .iter()
                .any(|t| t.function.name == def.function.name)
            {
                state.add_tool(def);
            }
        }
    }

    /// Returns a reference to the inner agent.
    ///
    /// 返回内部 Agent 的引用。
    pub fn agent(&self) -> &A {
        &self.agent
    }

    /// Returns a reference to the tool registry.
    ///
    /// 返回工具注册表的引用。
    pub fn registry(&self) -> &R {
        &self.registry
    }

    /// Returns a reference to the executor.
    ///
    /// 返回工具执行器的引用。
    pub fn executor(&self) -> &E {
        &self.executor
    }

    /// Returns the configured maximum turns.
    ///
    /// 返回配置的最大轮次数。
    pub fn max_turns(&self) -> usize {
        self.max_turns
    }
}

impl<A: ChatAgent, R: ToolRegistry, E: Executor<R>> Runner<A, R, E> {
    /// Runs the agent interaction loop synchronously/non-streamingly until completion or max turns reached.
    ///
    /// 非流式运行 Agent 交互循环和工具执行，直到完成或达到最大轮次数。
    pub async fn run(&self, state: &mut ConversationState) -> Result<Message, AgentError> {
        if self.auto_sync_tools {
            self.sync_tools(state);
        }

        let mut current_turn = 0;

        loop {
            if current_turn >= self.max_turns {
                let last_message = state
                    .messages()
                    .last()
                    .cloned()
                    .unwrap_or_else(|| Message::assistant(""));
                return Ok(last_message);
            }

            current_turn += 1;

            let msg = self.agent.chat(state.clone()).await?;
            state.add_message(msg.clone());

            let tool_calls: Vec<ToolCall> = msg
                .content
                .iter()
                .filter_map(|part| match part {
                    ContentPart::ToolCall(tc) => Some(tc.clone()),
                    _ => None,
                })
                .collect();

            if tool_calls.is_empty() {
                return Ok(msg);
            }

            match self.executor.execute(&self.registry, tool_calls).await {
                Ok(results) => {
                    state.add_message(Message {
                        role: Role::Tool,
                        content: results.into_iter().map(ContentPart::ToolResult).collect(),
                        name: None,
                    });
                }
                Err(err) => {
                    return Err(AgentError::ToolExecution(format!(
                        "Fatal error executing tool: {}",
                        err
                    )));
                }
            }
        }
    }

    /// Creates a stream that manages the agent interaction loop and tool execution.
    ///
    /// 创建管理 Agent 交互循环和工具执行的流。
    pub fn run_stream<'a>(&'a self, state: &'a mut ConversationState) -> RunnerStream<'a, A, R, E> {
        self.run_stream_with(state, crate::ChatStreamConfig::default)
    }

    /// Creates a stream that manages the agent interaction loop and tool execution with stream configuration hooks.
    ///
    /// 创建带有流配置 Hook 的管理 Agent 交互循环和工具执行的流。
    pub fn run_stream_with<'a, F>(
        &'a self,
        state: &'a mut ConversationState,
        config_fn: F,
    ) -> RunnerStream<'a, A, R, E>
    where
        F: Fn() -> crate::ChatStreamConfig<A::RawDelta> + Send + Sync + 'a,
    {
        if self.auto_sync_tools {
            self.sync_tools(state);
        }
        RunnerStream::new(
            &self.agent,
            &self.registry,
            &self.executor,
            state,
            self.max_turns,
            Some(Box::new(config_fn)),
        )
    }
}

/// A stream that manages the agent interaction loop, including tool execution.
///
/// 管理 Agent 交互循环（包含工具执行）的流。
pub struct RunnerStream<
    'a,
    A: ChatAgent + ?Sized + 'a,
    R: ToolRegistry = (),
    E: Executor<R> = DefaultExecutor,
> {
    agent: &'a A,
    registry: &'a R,
    executor: &'a E,
    state: &'a mut ConversationState,
    max_turns: usize,
    config_fn: Option<Box<dyn Fn() -> crate::ChatStreamConfig<A::RawDelta> + Send + Sync + 'a>>,

    phase: Phase<'a, A, R, E>,
    current_turn: usize,
    collected_events: Vec<ChatStreamEvent>,
}

enum Phase<'a, A: ChatAgent + ?Sized + 'a, R: ToolRegistry, E: Executor<R>> {
    Start,
    Initializing(A::ChatStreamFuture<'a>),
    Streaming(Box<ChatStream<A::Stream, AgentError>>),
    ExecutingTools(E::Future<'a>),
    Done,
}

impl<'a, A: ChatAgent + ?Sized + 'a, R: ToolRegistry, E: Executor<R>> RunnerStream<'a, A, R, E> {
    /// Creates a new `RunnerStream`.
    ///
    /// 创建一个新的 `RunnerStream`。
    pub fn new(
        agent: &'a A,
        registry: &'a R,
        executor: &'a E,
        state: &'a mut ConversationState,
        max_turns: usize,
        config_fn: Option<Box<dyn Fn() -> crate::ChatStreamConfig<A::RawDelta> + Send + Sync + 'a>>,
    ) -> Self {
        RunnerStream {
            agent,
            registry,
            executor,
            state,
            max_turns,
            config_fn,
            phase: Phase::Start,
            current_turn: 0,
            collected_events: Vec::new(),
        }
    }
}

impl<'a, A, R, E> Stream for RunnerStream<'a, A, R, E>
where
    A: ChatAgent + ?Sized + 'a,
    A::Stream: Unpin,
    A::ChatStreamFuture<'a>: Unpin,
    R: ToolRegistry,
    E: Executor<R>,
{
    type Item = Result<ChatStreamEvent, AgentError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = unsafe { self.get_unchecked_mut() };
        loop {
            match &mut this.phase {
                Phase::Start => {
                    if this.current_turn >= this.max_turns {
                        this.phase = Phase::Done;
                        return Poll::Ready(None);
                    }

                    this.current_turn += 1;
                    this.collected_events.clear();

                    let state_clone = this.state.clone();
                    let fut = if let Some(ref config_fn) = this.config_fn {
                        this.agent.chat_stream_with(state_clone, config_fn())
                    } else {
                        this.agent.chat_stream(state_clone)
                    };
                    this.phase = Phase::Initializing(fut);
                }
                Phase::Initializing(fut) => match Pin::new(fut).poll(cx) {
                    Poll::Ready(Ok(stream)) => {
                        this.phase = Phase::Streaming(Box::new(stream));
                    }
                    Poll::Ready(Err(e)) => {
                        this.phase = Phase::Done;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Phase::Streaming(stream) => match Pin::new(stream).poll_next(cx) {
                    Poll::Ready(Some(Ok(event))) => {
                        this.collected_events.push(event.clone());
                        return Poll::Ready(Some(Ok(event)));
                    }
                    Poll::Ready(Some(Err(e))) => {
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Ready(None) => {
                        let message: Message = this.collected_events.drain(..).collect();
                        this.state.add_message(message.clone());

                        let tool_calls: Vec<ToolCall> = message
                            .content
                            .into_iter()
                            .filter_map(|part| match part {
                                ContentPart::ToolCall(tc) => Some(tc),
                                _ => None,
                            })
                            .collect();

                        if tool_calls.is_empty() {
                            this.phase = Phase::Done;
                            return Poll::Ready(None);
                        }

                        let exec_fut = this.executor.execute(this.registry, tool_calls);
                        this.phase = Phase::ExecutingTools(exec_fut);
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Phase::ExecutingTools(fut) => {
                    let pinned_fut = unsafe { Pin::new_unchecked(fut) };
                    match pinned_fut.poll(cx) {
                        Poll::Ready(Ok(results)) => {
                            this.state.add_message(Message {
                                role: Role::Tool,
                                content: results.into_iter().map(ContentPart::ToolResult).collect(),
                                name: None,
                            });

                            this.phase = Phase::Start;
                        }
                        Poll::Ready(Err(err)) => {
                            this.phase = Phase::Done;
                            return Poll::Ready(Some(Err(AgentError::ToolExecution(format!(
                                "Fatal error executing tool: {}",
                                err
                            )))));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::Done => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_core::tool::{
        DynToolRegistry, ExecuteToolsFuture, JSONSchema, Schema, Tool, ToolDefinition, ToolError,
        ToolExecutionError,
    };
    use std::sync::Arc;

    #[derive(Clone)]
    struct DummyTool;

    impl ToolRunnable for DummyTool {
        type Error = String;
        type Future = std::future::Ready<Result<Vec<ContentPart>, ToolError<String>>>;

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                r#type: oxide_llm_core::tool::ToolType::Function,
                function: oxide_llm_core::tool::FunctionDefinition {
                    name: "dummy_tool".into(),
                    description: Some("A dummy tool for testing".into()),
                    parameters: None,
                    strict: None,
                },
            }
        }

        fn run(&self, _args: serde_json::Value) -> Self::Future {
            std::future::ready(Ok(vec![]))
        }
    }

    struct DummyAgent;

    impl ChatAgent for DummyAgent {
        type RawConversationState = ConversationState;
        type RawMessage = Message;
        type RawDelta = crate::core::message::DeltaMessage;
        type RawStream =
            futures::stream::Empty<Result<crate::core::message::DeltaMessage, AgentError>>;
        type ChatStreamRawFuture<'a>
            = std::future::Ready<Result<Self::RawStream, AgentError>>
        where
            Self: 'a;
        type Stream =
            futures::stream::Empty<Result<crate::core::message::DeltaMessage, AgentError>>;
        type ChatStreamFuture<'a>
            = std::future::Ready<Result<ChatStream<Self::Stream, AgentError>, AgentError>>
        where
            Self: 'a;

        async fn chat_raw(
            &self,
            _state: Self::RawConversationState,
        ) -> Result<Self::RawMessage, AgentError> {
            Ok(Message::user("dummy"))
        }

        fn chat_stream_raw_with<'a>(
            &'a self,
            _state: Self::RawConversationState,
            _config: crate::ChatStreamRawConfig<Self::RawDelta>,
        ) -> Self::ChatStreamRawFuture<'a> {
            std::future::ready(Ok(futures::stream::empty()))
        }

        async fn chat(&self, _state: ConversationState) -> Result<Message, AgentError> {
            Ok(Message::user("dummy"))
        }

        fn chat_stream_with<'a>(
            &'a self,
            _state: ConversationState,
            _config: crate::ChatStreamConfig<Self::RawDelta>,
        ) -> Self::ChatStreamFuture<'a> {
            std::future::ready(Ok(ChatStream::new(futures::stream::empty())))
        }

        fn chat_stream<'a>(&'a self, _state: ConversationState) -> Self::ChatStreamFuture<'a> {
            std::future::ready(Ok(ChatStream::new(futures::stream::empty())))
        }
    }

    #[test]
    fn test_runner_no_implicit_sync_by_default() {
        let agent = DummyAgent;
        let registry = DynToolRegistry::new().with(DummyTool);
        let runner = Runner::new(agent)
            .with_registry(registry)
            .with_max_turns(10);

        assert_eq!(runner.max_turns(), 10);
        assert_eq!(runner.registry().definitions().len(), 1);

        let mut state = ConversationState::new();
        assert!(state.tools().is_empty());

        // Default run_stream does NOT implicitly sync tools to ConversationState
        let stream = runner.run_stream(&mut state);
        drop(stream);
        assert!(state.tools().is_empty());

        // Explicit sync_tools fills state.tools
        runner.sync_tools(&mut state);
        assert_eq!(state.tools().len(), 1);
        assert_eq!(state.tools()[0].function.name, "dummy_tool");
    }

    #[test]
    fn test_runner_opt_in_auto_sync() {
        let agent = DummyAgent;
        let registry = DynToolRegistry::new().with(DummyTool);
        let runner = Runner::new(agent)
            .with_registry(registry)
            .with_auto_sync_tools(true);

        let mut state = ConversationState::new();
        assert!(state.tools().is_empty());

        // Opted-in auto sync fills state.tools during run_stream
        let stream = runner.run_stream(&mut state);
        drop(stream);
        assert_eq!(state.tools().len(), 1);
    }

    #[test]
    fn test_runner_run_method() {
        let agent = DummyAgent;
        let registry = DynToolRegistry::new().with(DummyTool);
        let runner = Runner::new(agent)
            .with_registry(registry)
            .with_auto_sync_tools(true);

        let mut state = ConversationState::new();
        let res = futures::executor::block_on(runner.run(&mut state));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Message::user("dummy"));
        assert_eq!(state.tools().len(), 1);
        assert_eq!(state.messages().len(), 1);
    }

    #[test]
    fn test_runner_with_smart_pointers() {
        let agent = Arc::new(DummyAgent);
        let registry = DynToolRegistry::new().with(DummyTool);
        let runner = Runner::new(agent).with_registry(registry);

        let mut state = ConversationState::new();
        let stream = runner.run_stream(&mut state);
        drop(stream);
        assert!(state.tools().is_empty());

        runner.sync_tools(&mut state);
        assert_eq!(state.tools().len(), 1);
    }

    #[test]
    fn test_runner_run_stream_with() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let agent = DummyAgent;
        let runner = Runner::new(agent);
        let mut state = ConversationState::new();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        let stream = runner.run_stream_with(&mut state, move || {
            let c = count_clone.clone();
            crate::ChatStreamConfig::new().on_raw_delta(move |_raw| {
                c.fetch_add(1, Ordering::SeqCst);
            })
        });

        use futures::StreamExt;
        futures::executor::block_on(async {
            let mut s = stream;
            while s.next().await.is_some() {}
        });

        // DummyAgent produces an empty stream, but run_stream_with correctly constructs and executes.
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[derive(Clone)]
    struct FatalTool;

    impl ToolRunnable for FatalTool {
        type Error = String;
        type Future = std::future::Ready<Result<Vec<ContentPart>, ToolError<String>>>;

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                r#type: oxide_llm_core::tool::ToolType::Function,
                function: oxide_llm_core::tool::FunctionDefinition {
                    name: "fatal_tool".into(),
                    description: Some("A fatal tool for testing".into()),
                    parameters: None,
                    strict: None,
                },
            }
        }

        fn run(&self, _args: serde_json::Value) -> Self::Future {
            std::future::ready(Err(ToolError::Custom("database unreachable".to_string())))
        }

        fn handle_error(
            &self,
            err: ToolError<Self::Error>,
        ) -> Result<Vec<ContentPart>, Self::Error> {
            match err {
                ToolError::Custom(e) => Err(e),
                other => Err(other.to_string()),
            }
        }
    }

    #[test]
    fn test_fatal_tool_execution_error_exits_runner() {
        let registry = DynToolRegistry::new().with(FatalTool);
        let tool_call = ToolCall {
            id: "call_1".into(),
            name: "fatal_tool".into(),
            arguments: serde_json::json!({}),
            signature: None,
        };

        let exec_fut = ExecuteToolsFuture::new(&registry, vec![tool_call]);
        let res = futures::executor::block_on(exec_fut);
        assert!(res.is_err());
        if let Err(ToolExecutionError::Fatal(msg)) = res {
            assert!(msg.contains("database unreachable"));
        } else {
            panic!("Expected ToolExecutionError::Fatal");
        }
    }

    struct SortingExecutor;

    impl<R: ToolRegistry> Executor<R> for SortingExecutor {
        type Future<'a> = ExecuteToolsFuture<'a, R>;

        fn execute<'a>(
            &'a self,
            registry: &'a R,
            mut tool_calls: Vec<ToolCall>,
        ) -> Self::Future<'a> {
            tool_calls.sort_by(|a, b| a.name.cmp(&b.name));
            ExecuteToolsFuture::new(registry, tool_calls)
        }
    }

    #[test]
    fn test_custom_sorting_executor() {
        let agent = DummyAgent;
        let registry = DynToolRegistry::new().with(DummyTool);
        let runner = Runner::new(agent)
            .with_registry(registry)
            .with_executor(SortingExecutor);

        let mut state = ConversationState::new();
        let _stream = runner.run_stream(&mut state);
    }

    #[derive(serde::Deserialize)]
    struct TypedArgs {
        count: i32,
    }

    impl Schema for TypedArgs {
        fn json_schema() -> JSONSchema {
            JSONSchema::object().required_property("count", JSONSchema::integer())
        }
    }

    #[derive(Clone)]
    struct TypedTool;

    impl Tool for TypedTool {
        const NAME: &'static str = "typed_tool";
        type Args = TypedArgs;
        type Output = String;
        type Error = String;
        type Future = std::future::Ready<Result<Self::Output, Self::Error>>;

        fn run(&self, args: Self::Args) -> Self::Future {
            std::future::ready(Ok(format!("count: {}", args.count)))
        }

        fn handle_error(
            &self,
            err: ToolError<Self::Error>,
        ) -> Result<Vec<ContentPart>, Self::Error> {
            match err {
                ToolError::InvalidArguments(msg) => Ok(vec![ContentPart::Text {
                    text: format!("Custom invalid args handler: {}", msg),
                    signature: None,
                }]),
                other => Ok(vec![ContentPart::Text {
                    text: format!("Other error: {}", other),
                    signature: None,
                }]),
            }
        }
    }

    #[test]
    fn test_tool_invalid_arguments_handled_by_user() {
        let registry = DynToolRegistry::new().with(TypedTool);
        let tool_call = ToolCall {
            id: "call_bad_arg".into(),
            name: "typed_tool".into(),
            arguments: serde_json::json!({"count": "invalid_number"}),
            signature: None,
        };

        let exec_fut = ExecuteToolsFuture::new(&registry, vec![tool_call]);
        let res = futures::executor::block_on(exec_fut);
        assert!(res.is_ok());
        let results = res.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_error);
        if let ContentPart::Text { ref text, .. } = results[0].content[0] {
            assert!(text.contains("Custom invalid args handler:"));
        } else {
            panic!("Expected ContentPart::Text");
        }
    }
}
