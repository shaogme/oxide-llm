use crate::message::Message;
use crate::tool::{ToolChoice, ToolDefinition};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Conversation State.
///
/// 对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ConversationState {
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

impl ConversationState {
    /// Create a new ConversationState.
    ///
    /// 创建一个新的 ConversationState。
    pub fn new(system_prompt: impl Into<Option<StaticRefStr>>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
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

/// Raw Conversation State for generic or protocol-specific message, tool, and tool choice types.
///
/// 用于通用或特定协议的消息、工具及工具选择类型的底层原始对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RawConversationState<M, T = (), C = ()> {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<StaticRefStr>,

    /// Raw Message list.
    ///
    /// 原始消息列表。
    pub messages: Vec<M>,

    /// Available raw tools.
    ///
    /// 可用原始工具列表。
    pub tools: Vec<T>,

    /// Raw Tool choice preference.
    ///
    /// 原始工具选择偏好。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<C>,
}

impl<M, T, C> RawConversationState<M, T, C> {
    /// Create a new RawConversationState.
    ///
    /// 创建一个新的 RawConversationState。
    pub fn new(system_prompt: impl Into<Option<StaticRefStr>>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            messages: Vec::new(),
            tools: Vec::new(),
            tool_choice: None,
        }
    }

    /// Add a raw message to history.
    ///
    /// 添加一条原始消息到历史。
    pub fn add_message(&mut self, message: M) {
        self.messages.push(message);
    }

    /// Add a raw tool.
    ///
    /// 添加一个原始工具。
    pub fn add_tool(&mut self, tool: T) {
        self.tools.push(tool);
    }

    /// Add multiple raw tools.
    ///
    /// 添加多个原始工具。
    pub fn add_tools(&mut self, tools: Vec<T>) {
        self.tools.extend(tools);
    }

    /// Set raw tool choice.
    ///
    /// 设置原始工具选择。
    pub fn set_tool_choice(&mut self, tool_choice: C) {
        self.tool_choice = Some(tool_choice);
    }
}

