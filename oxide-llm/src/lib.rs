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
pub mod stream;
pub mod tool;

pub mod core {
    pub use oxide_llm_core::*;
}

pub mod macros {
    pub use oxide_llm_macros::tool;
}

pub mod reexports {
    pub use serde_json;
}

/// Trait for chat agents.
///
/// 聊天代理 Trait。
#[trait_morph::morph(Send)]
pub trait ChatAgent: Send + Sync {
    /// The stream type returned by `chat_stream`.
    ///
    /// `chat_stream` 返回的流类型。
    type Stream: futures::Stream<Item = Result<DeltaMessage>> + Send + 'static;

    /// Send a chat request.
    ///
    /// 发送聊天请求。
    async fn chat(&self, state: ConversationState) -> Result<Message>;

    /// Send a chat request and receive a stream of chunks.
    ///
    /// 发送聊天请求并接收流式响应。
    async fn chat_stream(
        &self,
        state: ConversationState,
    ) -> Result<ChatStream<Self::Stream, AgentError>>;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_agent_is_dyn() {
        fn _assert_dyn(_: Box<dyn DynChatAgent>) {}
    }
}
