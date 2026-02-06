use crate::message::Message;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};

/// Conversation State.
///
/// 对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<String>,

    /// Message list.
    ///
    /// 消息列表。
    pub messages: Vec<Message>,

    /// Available tools.
    ///
    /// 可用工具列表。
    pub tools: Vec<Tool>,
}

impl ConversationState {
    /// Create a new ConversationState.
    ///
    /// 创建一个新的 ConversationState。
    pub fn new(system_prompt: Option<String>) -> Self {
        Self {
            system_prompt,
            messages: Vec::new(),
            tools: Vec::new(),
        }
    }

    /// Add a message to history.
    ///
    /// 添加一条消息到历史。
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Add a tool.
    ///
    /// 添加一个工具。
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }
}
