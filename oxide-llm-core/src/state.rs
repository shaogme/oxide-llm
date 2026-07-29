use std::borrow::Cow;

use crate::message::Message;
use crate::tool::{ToolChoice, ToolDefinition};
use serde::{Deserialize, Serialize};

/// Conversation State.
///
/// 对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<Cow<'static, str>>,

    /// Message list.
    ///
    /// 消息列表。
    pub messages: Vec<Message>,

    /// Available tools.
    ///
    /// 可用工具列表。
    pub tools: Vec<ToolDefinition>,

    /// Tool choice preference.
    ///
    /// 工具选择偏好。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

impl ConversationState {
    /// Create a new ConversationState.
    ///
    /// 创建一个新的 ConversationState。
    pub fn new(system_prompt: Option<Cow<'static, str>>) -> Self {
        Self {
            system_prompt,
            messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: None,
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
    pub fn add_tool(&mut self, tool: ToolDefinition) {
        self.tools.push(tool);
    }

    /// Add multiple tools.
    ///
    /// 添加多个工具。
    pub fn add_tools(&mut self, tools: Vec<ToolDefinition>) {
        self.tools.extend(tools);
    }

    /// Set tool choice.
    ///
    /// 设置工具选择。
    pub fn set_tool_choice(&mut self, tool_choice: ToolChoice) {
        self.tool_choice = Some(tool_choice);
    }
}
