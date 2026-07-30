use crate::message::Message;
use crate::tool::{ToolChoice, ToolDefinition};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Conversation State.
///
/// 对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(transparent)]
pub struct ConversationState {
    raw: RawConversationState,
}

impl ConversationState {
    /// Create a new ConversationState.
    ///
    /// 创建一个新的 ConversationState。
    pub fn new() -> Self {
        Self {
            raw: RawConversationState::default(),
        }
    }

    /// Create a new ConversationState with a system prompt.
    ///
    /// 创建一个带系统提示词的 ConversationState。
    pub fn with_system_prompt(system_prompt: impl Into<StaticRefStr>) -> Self {
        Self {
            raw: RawConversationState {
                system_prompt: Some(system_prompt.into()),
                ..Default::default()
            },
        }
    }

    /// Get the system prompt.
    ///
    /// 获取系统提示词。
    pub fn system_prompt(&self) -> Option<&StaticRefStr> {
        self.raw.system_prompt.as_ref()
    }

    /// Set system prompt.
    ///
    /// 设置系统提示词。
    pub fn set_system_prompt(&mut self, system_prompt: impl Into<StaticRefStr>) -> &mut Self {
        self.raw.system_prompt = Some(system_prompt.into());
        self
    }

    /// Clear system prompt.
    ///
    /// 清除系统提示词。
    pub fn clear_system_prompt(&mut self) -> &mut Self {
        self.raw.system_prompt = None;
        self
    }

    /// Get the messages slice.
    ///
    /// 获取消息列表只读切片。
    pub fn messages(&self) -> &[Message] {
        &self.raw.messages
    }

    /// Get mutable reference to messages.
    ///
    /// 获取消息列表的可变引用。
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.raw.messages
    }

    /// Set the messages.
    ///
    /// 设置消息列表。
    pub fn set_messages(&mut self, messages: Vec<Message>) -> &mut Self {
        self.raw.messages = messages;
        self
    }

    /// Add a message to history.
    ///
    /// 添加一条消息到历史。
    pub fn add_message(&mut self, message: Message) {
        self.raw.messages.push(message);
    }

    /// Clear messages.
    ///
    /// 清除消息列表。
    pub fn clear_messages(&mut self) -> &mut Self {
        self.raw.messages.clear();
        self
    }

    /// Consume state and return messages.
    ///
    /// 消耗状态并返回消息列表。
    pub fn into_messages(self) -> Vec<Message> {
        self.raw.messages
    }

    /// Get the tools slice.
    ///
    /// 获取工具列表只读切片。
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.raw.tools
    }

    /// Get mutable reference to tools.
    ///
    /// 获取工具列表的可变引用。
    pub fn tools_mut(&mut self) -> &mut Vec<ToolDefinition> {
        &mut self.raw.tools
    }

    /// Set the tools.
    ///
    /// 设置工具列表。
    pub fn set_tools(&mut self, tools: Vec<ToolDefinition>) -> &mut Self {
        self.raw.tools = tools;
        self
    }

    /// Add a tool.
    ///
    /// 添加一个工具。
    pub fn add_tool(&mut self, tool: ToolDefinition) {
        self.raw.tools.push(tool);
    }

    /// Add multiple tools.
    ///
    /// 添加多个工具。
    pub fn add_tools(&mut self, tools: Vec<ToolDefinition>) {
        self.raw.tools.extend(tools);
    }

    /// Clear tools.
    ///
    /// 清除工具列表。
    pub fn clear_tools(&mut self) -> &mut Self {
        self.raw.tools.clear();
        self
    }

    /// Consume state and return tools.
    ///
    /// 消耗状态并返回工具列表。
    pub fn into_tools(self) -> Vec<ToolDefinition> {
        self.raw.tools
    }

    /// Get the tool choice.
    ///
    /// 获取工具选择偏好。
    pub fn tool_choice(&self) -> Option<&ToolChoice> {
        self.raw.tool_choice.as_ref()
    }

    /// Set tool choice.
    ///
    /// 设置工具选择。
    pub fn set_tool_choice(&mut self, tool_choice: ToolChoice) {
        self.raw.tool_choice = Some(tool_choice);
    }

    /// Clear tool choice.
    ///
    /// 清除工具选择偏好。
    pub fn clear_tool_choice(&mut self) -> &mut Self {
        self.raw.tool_choice = None;
        self
    }

    /// Convert into raw conversation state.
    ///
    /// 转换为原始对话状态。
    pub fn into_raw(self) -> RawConversationState {
        self.raw
    }

    /// Create ConversationState from raw conversation state.
    ///
    /// 从原始对话状态创建 ConversationState。
    pub fn from_raw(raw: RawConversationState) -> Self {
        Self { raw }
    }
}

/// Raw conversation state with public fields.
///
/// 带有公开字段的原始对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RawConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<StaticRefStr>,

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

impl From<ConversationState> for RawConversationState {
    fn from(state: ConversationState) -> Self {
        state.into_raw()
    }
}

impl From<RawConversationState> for ConversationState {
    fn from(raw: RawConversationState) -> Self {
        ConversationState::from_raw(raw)
    }
}

/// Trait for conversation states (either core or protocol-specific).
///
/// 对话状态 Trait (通用或特定协议)。
pub trait ConversationStateTrait: Send + 'static {}

impl ConversationStateTrait for ConversationState {}
impl ConversationStateTrait for RawConversationState {}
