use crate::ChatAgent;
use crate::core::message::{ChatStream, ChatStreamEvent, ContentPart, Message, Role};
use crate::core::state::ConversationState;
use crate::core::tool::{ToolCall, ToolFuture, ToolResult, ToolRunnable};
use crate::error::AgentError;
use crate::tool::ToolRegistry;
use futures::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// High-level runner for orchestrating agent interactions and tool executions.
///
/// 用于协调 Agent 交互和工具执行的高级 Runner。
#[derive(Clone, Default)]
pub struct Runner<A> {
    agent: A,
    registry: ToolRegistry,
    max_turns: usize,
    auto_sync_tools: bool,
}

impl<A> Runner<A> {
    /// Creates a new `Runner` wrapping the given agent with default max turns (5).
    ///
    /// 创建一个新的 `Runner` 包装指定的 Agent，默认最大轮次数为 5。
    pub fn new(agent: A) -> Self {
        Self {
            agent,
            registry: ToolRegistry::new(),
            max_turns: 5,
            auto_sync_tools: false,
        }
    }

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

    /// Registers a tool with the runner.
    ///
    /// 向 Runner 注册一个工具。
    pub fn with_tool<T>(mut self, tool: T) -> Self
    where
        T: ToolRunnable + 'static,
    {
        self.registry.register(tool);
        self
    }

    /// Sets the tool registry for the runner.
    ///
    /// 设置 Runner 的工具注册表。
    pub fn with_registry(mut self, registry: ToolRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Synchronizes registered tools into the conversation state.
    ///
    /// 将已注册的工具同步填充到对话状态中。
    pub fn sync_tools(&self, state: &mut ConversationState) {
        let definitions = self.registry.definitions();
        for def in definitions {
            if !state
                .tools
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
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Returns the configured maximum turns.
    ///
    /// 返回配置的最大轮次数。
    pub fn max_turns(&self) -> usize {
        self.max_turns
    }
}

impl<A: ChatAgent> Runner<A> {
    /// Creates a stream that manages the agent interaction loop and tool execution.
    ///
    /// 创建管理 Agent 交互循环和工具执行的流。
    pub fn run_stream<'a>(&'a self, state: &'a mut ConversationState) -> RunnerStream<'a, A> {
        if self.auto_sync_tools {
            self.sync_tools(state);
        }
        RunnerStream::new(&self.agent, &self.registry, state, self.max_turns)
    }
}

/// Future that executes a series of tool calls sequentially.
///
/// 顺序执行一系列工具调用的 Future。
pub struct ExecuteToolsFuture {
    registry: ToolRegistry,
    tool_calls: std::vec::IntoIter<ToolCall>,
    current_exec: Option<(ToolCall, ToolFuture)>,
    results: Vec<ToolResult>,
}

impl ExecuteToolsFuture {
    /// Creates a new `ExecuteToolsFuture`.
    ///
    /// 创建一个新的 `ExecuteToolsFuture`。
    pub fn new(registry: ToolRegistry, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            registry,
            tool_calls: tool_calls.into_iter(),
            current_exec: None,
            results: Vec::new(),
        }
    }
}

impl Unpin for ExecuteToolsFuture {}

impl Future for ExecuteToolsFuture {
    type Output = Vec<ToolResult>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        loop {
            if let Some((_, ref mut fut)) = this.current_exec {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(res_bucket) => {
                        let (tool_call, _) = this.current_exec.take().unwrap();
                        let result = match res_bucket {
                            Ok(content) => ToolResult {
                                tool_call_id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                content,
                                is_error: false,
                                signature: tool_call.signature.clone(),
                            },
                            Err(err) => ToolResult {
                                tool_call_id: tool_call.id.clone(),
                                name: tool_call.name.clone(),
                                content: vec![ContentPart::Text {
                                    text: format!("Error executing tool: {}", err).into(),
                                    signature: None,
                                }],
                                is_error: true,
                                signature: tool_call.signature.clone(),
                            },
                        };
                        this.results.push(result);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            if let Some(tool_call) = this.tool_calls.next() {
                if let Some(fut) = this
                    .registry
                    .execute_future(&tool_call.name, tool_call.arguments.clone())
                {
                    this.current_exec = Some((tool_call, fut));
                } else {
                    let result = ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        content: vec![ContentPart::Text {
                            text: format!("Error: Unknown tool '{}'", tool_call.name).into(),
                            signature: None,
                        }],
                        is_error: true,
                        signature: tool_call.signature.clone(),
                    };
                    this.results.push(result);
                }
            } else {
                return Poll::Ready(std::mem::take(&mut this.results));
            }
        }
    }
}

/// A stream that manages the agent interaction loop, including tool execution.
///
/// 管理 Agent 交互循环（包含工具执行）的流。
pub struct RunnerStream<'a, A: ChatAgent + ?Sized + 'a> {
    agent: &'a A,
    registry: &'a ToolRegistry,
    state: &'a mut ConversationState,
    max_turns: usize,

    phase: Phase<'a, A>,
    current_turn: usize,
    collected_events: Vec<ChatStreamEvent>,
}

enum Phase<'a, A: ChatAgent + ?Sized + 'a> {
    Start,
    Initializing(A::ChatStreamFuture<'a>),
    Streaming(ChatStream<A::Stream, AgentError>),
    ExecutingTools(ExecuteToolsFuture),
    Done,
}

impl<'a, A: ChatAgent + ?Sized + 'a> RunnerStream<'a, A> {
    /// Creates a new `RunnerStream`.
    ///
    /// 创建一个新的 `RunnerStream`。
    pub fn new(
        agent: &'a A,
        registry: &'a ToolRegistry,
        state: &'a mut ConversationState,
        max_turns: usize,
    ) -> Self {
        RunnerStream {
            agent,
            registry,
            state,
            max_turns,
            phase: Phase::Start,
            current_turn: 0,
            collected_events: Vec::new(),
        }
    }
}

impl<'a, A> Stream for RunnerStream<'a, A>
where
    A: ChatAgent + ?Sized + 'a,
    A::Stream: Unpin,
    A::ChatStreamFuture<'a>: Unpin,
{
    type Item = Result<ChatStreamEvent, AgentError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
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
                    let fut = this.agent.chat_stream(state_clone);
                    this.phase = Phase::Initializing(fut);
                }
                Phase::Initializing(fut) => match Pin::new(fut).poll(cx) {
                    Poll::Ready(Ok(stream)) => {
                        this.phase = Phase::Streaming(stream);
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

                        let exec_fut = ExecuteToolsFuture::new(this.registry.clone(), tool_calls);
                        this.phase = Phase::ExecutingTools(exec_fut);
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Phase::ExecutingTools(fut) => match Pin::new(fut).poll(cx) {
                    Poll::Ready(results) => {
                        this.state.add_message(Message {
                            role: Role::Tool,
                            content: results.into_iter().map(ContentPart::ToolResult).collect(),
                            name: None,
                        });

                        this.phase = Phase::Start;
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Phase::Done => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_core::tool::ToolDefinition;
    use std::sync::Arc;

    struct DummyTool;

    impl ToolRunnable for DummyTool {
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

        fn run(&self, _args: serde_json::Value) -> oxide_llm_core::tool::ToolFuture {
            Box::pin(async { Ok(vec![]) })
        }
    }

    struct DummyAgent;

    impl ChatAgent for DummyAgent {
        type Stream =
            futures::stream::Empty<Result<crate::core::message::DeltaMessage, AgentError>>;
        type ChatStreamFuture<'a>
            = std::future::Ready<Result<ChatStream<Self::Stream, AgentError>, AgentError>>
        where
            Self: 'a;

        async fn chat(&self, _state: ConversationState) -> Result<Message, AgentError> {
            Ok(Message::user("dummy"))
        }

        fn chat_stream<'a>(&'a self, _state: ConversationState) -> Self::ChatStreamFuture<'a> {
            std::future::ready(Ok(ChatStream::new(futures::stream::empty())))
        }
    }

    #[test]
    fn test_runner_no_implicit_sync_by_default() {
        let agent = DummyAgent;
        let runner = Runner::new(agent).with_tool(DummyTool).with_max_turns(10);

        assert_eq!(runner.max_turns(), 10);
        assert_eq!(runner.registry().definitions().len(), 1);

        let mut state = ConversationState::new(None);
        assert!(state.tools.is_empty());

        // Default run_stream does NOT implicitly sync tools to ConversationState
        let _stream = runner.run_stream(&mut state);
        assert!(state.tools.is_empty());

        // Explicit sync_tools fills state.tools
        runner.sync_tools(&mut state);
        assert_eq!(state.tools.len(), 1);
        assert_eq!(state.tools[0].function.name, "dummy_tool");
    }

    #[test]
    fn test_runner_opt_in_auto_sync() {
        let agent = DummyAgent;
        let runner = Runner::new(agent)
            .with_tool(DummyTool)
            .with_auto_sync_tools(true);

        let mut state = ConversationState::new(None);
        assert!(state.tools.is_empty());

        // Opted-in auto sync fills state.tools during run_stream
        let _stream = runner.run_stream(&mut state);
        assert_eq!(state.tools.len(), 1);
    }

    #[test]
    fn test_runner_with_smart_pointers() {
        let agent = Arc::new(DummyAgent);
        let runner = Runner::new(agent).with_tool(DummyTool);

        let mut state = ConversationState::new(None);
        let _stream = runner.run_stream(&mut state);
        assert!(state.tools.is_empty());

        runner.sync_tools(&mut state);
        assert_eq!(state.tools.len(), 1);
    }
}
