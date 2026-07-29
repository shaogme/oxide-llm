use crate::error::{AgentError, Result};
use futures::{future::BoxFuture, stream::BoxStream};
use oxide_llm_core::{
    message::{ChatStream, DeltaMessage, Message},
    state::ConversationState,
};

pub mod agent {
    pub mod claude;
    pub mod gemini;
    pub mod openai;
}

pub mod error;
pub mod runner;
pub mod stream;

pub use runner::{DefaultExecutor, Executor, Runner, SequentialExecutor};

pub mod macros {
    pub use oxide_llm_macros::*;
}

#[cfg(feature = "transport")]
pub mod transport {
    #[cfg(feature = "transport-reqwest")]
    pub use oxide_llm_transport::reqwest;
}

pub mod core {
    pub use oxide_llm_core::*;
}

pub mod reexports {
    pub use serde_json;
}

/// Trait for chat agents.
///
/// 聊天代理 Trait。
#[trait_morph::morph(Send)]
pub trait ChatAgent: Send + Sync {
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
    async fn chat_raw(&self, state: ConversationState) -> Result<Self::RawMessage>;

    /// Send a chat request and receive a stream of raw chunks.
    ///
    /// 发送聊天请求并接收底层原始块的流式响应。
    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a>;

    /// Send a chat request.
    ///
    /// 发送聊天请求。
    async fn chat(&self, state: ConversationState) -> Result<Message>;

    /// Send a chat request and receive a stream of chunks.
    ///
    /// 发送聊天请求并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a>;
}

impl<T: ChatAgent + ?Sized> ChatAgent for &T {
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

    async fn chat_raw(&self, state: ConversationState) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream(state)
    }
}

impl<T: ChatAgent + ?Sized> ChatAgent for Box<T> {
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

    async fn chat_raw(&self, state: ConversationState) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
    }

    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        (**self).chat_stream(state)
    }
}

impl<T: ChatAgent + ?Sized> ChatAgent for std::sync::Arc<T> {
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

    async fn chat_raw(&self, state: ConversationState) -> Result<Self::RawMessage> {
        (**self).chat_raw(state).await
    }

    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        (**self).chat_stream_raw(state)
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        (**self).chat(state).await
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

    async fn chat_raw(&self, state: ConversationState) -> Result<Self::RawMessage> {
        DynChatAgent::chat(self, state).await
    }

    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        Box::pin(async move {
            let stream = DynChatAgent::chat_stream(self, state).await?;
            Ok(stream.into_inner())
        })
    }

    async fn chat(&self, state: ConversationState) -> Result<Message> {
        DynChatAgent::chat(self, state).await
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
}
