use crate::{
    config::{ChatStreamConfig, ChatStreamRawConfig},
    error::{AgentError, Result},
};
use futures::{future::BoxFuture, stream::BoxStream};
use oxide_llm_core::{
    message::{ChatStream, DeltaMessage, Message},
    state::{ConversationState, RawConversationState},
};

/// Trait for chat agents.
///
/// 聊天代理 Trait。
#[trait_morph::morph(Send)]
pub trait ChatAgent: Send + Sync {
    /// The raw input message type used in `RawConversationState`.
    ///
    /// `RawConversationState` 中使用的底层原始输入消息类型。
    type RawInputMessage: Send + 'static;

    /// The raw tool definition type used in `RawConversationState`.
    ///
    /// `RawConversationState` 中使用的底层原始工具类型。
    type RawTool: Send + 'static;

    /// The raw tool choice type used in `RawConversationState`.
    ///
    /// `RawConversationState` 中使用的底层原始工具选择类型。
    type RawToolChoice: Send + 'static;

    /// The raw message type returned by `chat_raw`.
    ///
    /// `chat_raw` 返回的底层裸消息类型。
    type RawMessage: Send + 'static;

    /// The raw delta message type returned by `RawStream`.
    ///
    /// `RawStream` 产出的底层裸增量消息类型。
    type RawDelta: Send + 'static;

    /// The raw stream type returned by `chat_stream_raw`.
    ///
    /// `chat_stream_raw` 返回的底层流类型。
    type RawStream: futures::Stream<Item = Result<Self::RawDelta>> + Send + 'static;

    /// The future type returned by `chat_stream_raw`.
    ///
    /// `chat_stream_raw` 返回的 Future 类型。
    type ChatStreamRawFuture<'a>: std::future::Future<Output = Result<Self::RawStream>>
        + Send
        + 'a
    where
        Self: 'a;

    /// The stream type returned by `chat_stream`.
    ///
    /// `chat_stream` 返回的流类型。
    type Stream: futures::Stream<Item = Result<DeltaMessage>> + Send + 'static;

    /// The future type returned by `chat_stream`.
    ///
    /// `chat_stream` 返回的 Future 类型。
    type ChatStreamFuture<'a>: std::future::Future<Output = Result<ChatStream<Self::Stream, AgentError>>>
        + Send
        + 'a
    where
        Self: 'a;

    /// Send a chat request and return the raw response message.
    ///
    /// 发送聊天请求并返回底层原始响应消息。
    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<Self::RawMessage>;

    /// Send a chat request and receive a stream of raw chunks with configuration.
    ///
    /// 发送聊天请求并接收带有配置的底层原始块的流式响应。
    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        config: ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a>;

    /// Send a chat request and receive a stream of raw chunks.
    ///
    /// 发送聊天请求并接收底层原始块的流式响应。
    fn chat_stream_raw<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Self::ChatStreamRawFuture<'a> {
        self.chat_stream_raw_with(state, ChatStreamRawConfig::default())
    }

    /// Send a chat request.
    ///
    /// 发送聊天请求。
    async fn chat(&self, state: ConversationState) -> Result<Message>;

    /// Send a chat request and receive a stream of chunks with configuration.
    ///
    /// 发送聊天请求并接收带有配置的流式响应。
    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        config: ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a>;

    /// Send a chat request and receive a stream of chunks.
    ///
    /// 发送聊天请求并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        self.chat_stream_with(state, ChatStreamConfig::default())
    }
}

impl<T: ChatAgent + ?Sized> ChatAgent for &T {
    type RawInputMessage = T::RawInputMessage;
    type RawTool = T::RawTool;
    type RawToolChoice = T::RawToolChoice;
    type RawMessage = T::RawMessage;
    type RawDelta = T::RawDelta;
    type RawStream = T::RawStream;
    type ChatStreamRawFuture<'a>
        = T::ChatStreamRawFuture<'a>
    where
        Self: 'a;
    type Stream = T::Stream;
    type ChatStreamFuture<'a>
        = T::ChatStreamFuture<'a>
    where
        Self: 'a;

    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        config: ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw_with(state, config)
    }

    fn chat_stream_raw<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
    }

    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        config: ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream_with(state, config)
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream(state)
    }
}

impl<T: ChatAgent + ?Sized> ChatAgent for Box<T> {
    type RawInputMessage = T::RawInputMessage;
    type RawTool = T::RawTool;
    type RawToolChoice = T::RawToolChoice;
    type RawMessage = T::RawMessage;
    type RawDelta = T::RawDelta;
    type RawStream = T::RawStream;
    type ChatStreamRawFuture<'a>
        = T::ChatStreamRawFuture<'a>
    where
        Self: 'a;
    type Stream = T::Stream;
    type ChatStreamFuture<'a>
        = T::ChatStreamFuture<'a>
    where
        Self: 'a;

    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        config: ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw_with(state, config)
    }

    fn chat_stream_raw<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
    }

    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        config: ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream_with(state, config)
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream(state)
    }
}

impl<T: ChatAgent + ?Sized> ChatAgent for std::sync::Arc<T> {
    type RawInputMessage = T::RawInputMessage;
    type RawTool = T::RawTool;
    type RawToolChoice = T::RawToolChoice;
    type RawMessage = T::RawMessage;
    type RawDelta = T::RawDelta;
    type RawStream = T::RawStream;
    type ChatStreamRawFuture<'a>
        = T::ChatStreamRawFuture<'a>
    where
        Self: 'a;
    type Stream = T::Stream;
    type ChatStreamFuture<'a>
        = T::ChatStreamFuture<'a>
    where
        Self: 'a;

    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        config: ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw_with(state, config)
    }

    fn chat_stream_raw<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
    }

    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        config: ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream_with(state, config)
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream(state)
    }
}

/// Trait for chat agents (Dynamic Dispatch Version).
///
/// 聊天代理 Trait (动态分发版本)。
pub trait DynChatAgent: Send + Sync {
    /// Send a chat request.
    ///
    /// 发送聊天请求。
    fn chat<'a>(&'a self, state: ConversationState) -> BoxFuture<'a, Result<Message>>;

    /// Send a chat request and receive a stream of chunks.
    ///
    /// 发送聊天请求并接收流式响应。
    fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> BoxFuture<'a, Result<ChatStream<BoxStream<'static, Result<DeltaMessage>>, AgentError>>>;
}

impl<T: ChatAgent> DynChatAgent for T {
    fn chat<'a>(&'a self, state: ConversationState) -> BoxFuture<'a, Result<Message>> {
        Box::pin(async move { self.chat(state).await })
    }

    fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> BoxFuture<'a, Result<ChatStream<BoxStream<'static, Result<DeltaMessage>>, AgentError>>>
    {
        Box::pin(async move {
            let stream = self.chat_stream(state).await?;
            Ok(stream.into_boxed())
        })
    }
}

impl ChatAgent for dyn DynChatAgent + '_ {
    type RawInputMessage = Message;
    type RawTool = oxide_llm_core::tool::ToolDefinition;
    type RawToolChoice = oxide_llm_core::tool::ToolChoice;
    type RawMessage = Message;
    type RawDelta = DeltaMessage;
    type RawStream = BoxStream<'static, Result<DeltaMessage>>;
    type ChatStreamRawFuture<'a>
        = BoxFuture<'a, Result<Self::RawStream>>
    where
        Self: 'a;
    type Stream = BoxStream<'static, Result<DeltaMessage>>;
    type ChatStreamFuture<'a>
        = BoxFuture<'a, Result<ChatStream<Self::Stream, AgentError>>>
    where
        Self: 'a;

    async fn chat_raw(
        &self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Result<Self::RawMessage> {
        let core_state = ConversationState {
            system_prompt: state.system_prompt,
            messages: state.messages,
            tools: state.tools,
            tool_choice: state.tool_choice,
        };
        DynChatAgent::chat(self, core_state).await
    }

    fn chat_stream_raw_with<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
        _config: ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        Box::pin(async move {
            let core_state = ConversationState {
                system_prompt: state.system_prompt,
                messages: state.messages,
                tools: state.tools,
                tool_choice: state.tool_choice,
            };
            let stream = DynChatAgent::chat_stream(self, core_state).await?;
            Ok(stream.into_inner())
        })
    }

    fn chat_stream_raw<'a>(
        &'a self,
        state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
    ) -> Self::ChatStreamRawFuture<'a> {
        Box::pin(async move {
            let core_state = ConversationState {
                system_prompt: state.system_prompt,
                messages: state.messages,
                tools: state.tools,
                tool_choice: state.tool_choice,
            };
            let stream = DynChatAgent::chat_stream(self, core_state).await?;
            Ok(stream.into_inner())
        })
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        DynChatAgent::chat(self, state).await
    }

    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        _config: ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        DynChatAgent::chat_stream(self, state)
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        DynChatAgent::chat_stream(self, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_agent_is_dyn() {
        fn _assert_dyn(_: Box<dyn DynChatAgent>) {}
    }

    #[test]
    fn test_chat_stream_hooks() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        struct TestAgent;
        impl ChatAgent for TestAgent {
            type RawInputMessage = Message;
            type RawTool = oxide_llm_core::tool::ToolDefinition;
            type RawToolChoice = oxide_llm_core::tool::ToolChoice;
            type RawMessage = Message;
            type RawDelta = String;
            type RawStream = crate::stream::RawHookStream<
                futures::stream::Iter<std::vec::IntoIter<Result<String>>>,
                String,
            >;
            type ChatStreamRawFuture<'a> = std::future::Ready<Result<Self::RawStream>>;
            type Stream = crate::stream::MappedStream<
                futures::stream::Iter<std::vec::IntoIter<Result<String>>>,
                TestMapper,
                String,
            >;
            type ChatStreamFuture<'a> = std::future::Ready<Result<ChatStream<Self::Stream, AgentError>>>;

            async fn chat_raw(
                &self,
                _state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
            ) -> Result<Self::RawMessage> {
                Ok(Message::user("test"))
            }

            fn chat_stream_raw_with<'a>(
                &'a self,
                _state: RawConversationState<Self::RawInputMessage, Self::RawTool, Self::RawToolChoice>,
                mut config: ChatStreamRawConfig<Self::RawDelta>,
            ) -> Self::ChatStreamRawFuture<'a> {
                let raw_stream = futures::stream::iter(vec![Ok("chunk1".to_string()), Ok("chunk2".to_string())]);
                let on_raw = config.take_on_raw_delta();
                let hook_stream = crate::stream::RawHookStream::new(raw_stream, on_raw);
                std::future::ready(Ok(hook_stream))
            }

            async fn chat(&self, _state: ConversationState) -> Result<Message> {
                Ok(Message::user("test"))
            }

            fn chat_stream_with<'a>(
                &'a self,
                _state: ConversationState,
                mut config: ChatStreamConfig<Self::RawDelta>,
            ) -> Self::ChatStreamFuture<'a> {
                let raw_stream = futures::stream::iter(vec![Ok("chunk1".to_string()), Ok("chunk2".to_string())]);
                let on_raw = config.take_on_raw_delta();
                let on_delta = config.take_on_delta();
                let mapped = crate::stream::MappedStream::with_hooks(raw_stream, TestMapper, on_raw, on_delta);
                std::future::ready(Ok(ChatStream::new(mapped)))
            }
        }

        struct TestMapper;
        impl crate::stream::StreamMapper<String> for TestMapper {
            fn map_item(&mut self, raw: String) -> Result<Option<DeltaMessage>> {
                Ok(Some(DeltaMessage {
                    role: None,
                    content: Some(vec![oxide_llm_core::message::DeltaContentPart::Text {
                        index: 0,
                        text: raw,
                        signature: None,
                    }]),
                    finish_reason: None,
                    name: None,
                    usage: None,
                }))
            }
        }

        futures::executor::block_on(async {
            let agent = TestAgent;

            // 1. Test chat_stream_raw_with
            let raw_count_1 = Arc::new(AtomicUsize::new(0));
            let raw_c1 = raw_count_1.clone();
            let raw_config = ChatStreamRawConfig::new().on_raw_delta(move |_raw| {
                raw_c1.fetch_add(1, Ordering::SeqCst);
            });

            use futures::StreamExt;
            let mut raw_stream = agent.chat_stream_raw_with(RawConversationState::new(None), raw_config).await.unwrap();
            while let Some(_) = raw_stream.next().await {}
            assert_eq!(raw_count_1.load(Ordering::SeqCst), 2);

            // 2. Test chat_stream_with
            let raw_count_2 = Arc::new(AtomicUsize::new(0));
            let delta_count_2 = Arc::new(AtomicUsize::new(0));

            let raw_c2 = raw_count_2.clone();
            let delta_c2 = delta_count_2.clone();

            let config = ChatStreamConfig::new()
                .on_raw_delta(move |_raw| {
                    raw_c2.fetch_add(1, Ordering::SeqCst);
                })
                .on_delta(move |_delta| {
                    delta_c2.fetch_add(1, Ordering::SeqCst);
                });

            let mut stream = agent.chat_stream_with(ConversationState::new(None), config).await.unwrap();
            while let Some(_) = stream.next().await {}

            assert_eq!(raw_count_2.load(Ordering::SeqCst), 2);
            assert_eq!(delta_count_2.load(Ordering::SeqCst), 2);
        });
    }
}
