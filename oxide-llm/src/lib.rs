use crate::error::{AgentError, Result};
use futures::future::BoxFuture;
use oxide_llm_core::{
    message::{ChatStreamWrapper, Message},
    state::ConversationState,
};

pub mod agent {
    pub mod claude;
    pub mod gemini;
    pub mod openai;
}

pub mod error;

pub mod core {
    pub use oxide_llm_core::*;
}

/// Trait for chat agents.
///
/// 聊天代理 Trait。
#[trait_morph::morph(Send)]
pub trait ChatAgent: Send + Sync {
    /// Send a chat request.
    ///
    /// 发送聊天请求。
    async fn chat(&self, state: ConversationState) -> Result<Message>;

    /// Send a chat request and receive a stream of chunks.
    ///
    /// 发送聊天请求并接收流式响应。
    async fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> Result<ChatStreamWrapper<'a, AgentError>>;
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
    ) -> BoxFuture<'a, Result<ChatStreamWrapper<'a, AgentError>>>;
}

impl<T: ChatAgent> DynChatAgent for T {
    fn chat<'a>(&'a self, state: ConversationState) -> BoxFuture<'a, Result<Message>> {
        Box::pin(async move { self.chat(state).await })
    }

    fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> BoxFuture<'a, Result<ChatStreamWrapper<'a, AgentError>>> {
        Box::pin(async move { self.chat_stream(state).await })
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
