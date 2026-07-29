use ref_str::StaticRefStr;
use std::collections::{BTreeMap, HashMap};

use crate::message::delta::{DeltaContentPart, DeltaMessage, FinishReason, Usage};
use crate::message::model::{ContentPart, Message, Role};
use crate::tool::ToolCall;

/// Helper struct to assemble a complete Message from DeltaMessages.
///
/// 用于将多个 DeltaMessage 组装成完整 Message 的辅助结构。
#[derive(Debug, Clone, Default)]
pub struct MessageAssembler {
    role: Option<Role>,
    name: Option<StaticRefStr>,

    // Text and Reasoning parts: indexed
    content_parts: BTreeMap<u32, AssembledPart>,

    // Tool calls: keyed by ID
    tool_calls: HashMap<StaticRefStr, AssembledToolCall>,

    // Optimization: Map index to the current active tool ID
    // This allows O(1) lookup for incoming tool call deltas that lack an ID.
    active_tool_id: HashMap<u32, StaticRefStr>,

    // Record appearance order: (index, id)
    tool_call_order: Vec<(u32, StaticRefStr)>,

    usage: Option<Usage>,
    finish_reason: Option<FinishReason>,
}

/// Assembled content part (Text and Reasoning only)
#[derive(Debug, Clone)]
enum AssembledPart {
    Text {
        text: String,
        signature: Option<StaticRefStr>,
    },
    Reasoning {
        text: String,
        signature: Option<StaticRefStr>,
    },
}

/// Assembled tool call
///
/// 已组装的工具调用
#[derive(Debug, Clone)]
struct AssembledToolCall {
    id: StaticRefStr,
    r#type: Option<StaticRefStr>,
    name: Option<StaticRefStr>,
    arguments: String,
    signature: Option<StaticRefStr>,
}

impl AssembledToolCall {
    fn to_tool_call(&self) -> ToolCall {
        let arguments: serde_json::Value = if self.arguments.trim().is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&self.arguments)
                .unwrap_or_else(|_| serde_json::Value::String(self.arguments.clone()))
        };

        ToolCall {
            id: self.id.clone(),
            name: self.name.clone().unwrap_or_default(),
            arguments,
            signature: self.signature.clone(),
        }
    }
}

impl MessageAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metadata fields (role, name, finish_reason, usage) from a DeltaMessage.
    ///
    /// 从 DeltaMessage 更新元数据字段 (role, name, finish_reason, usage)。
    pub fn add_metadata(&mut self, delta: &DeltaMessage) {
        if let Some(role) = delta.role {
            self.role = Some(role);
        }
        if let Some(name) = delta.name.as_ref() {
            self.name = Some(name.clone());
        }
        if let Some(reason) = delta.finish_reason.as_ref() {
            self.finish_reason = Some(reason.clone());
        }
        if let Some(usage) = delta.usage.as_ref() {
            if let Some(current) = self.usage.as_mut() {
                current.input_tokens = current.input_tokens.max(usage.input_tokens);
                current.output_tokens = current.output_tokens.max(usage.output_tokens);
                current.total_tokens = current.input_tokens + current.output_tokens;
            } else {
                self.usage = Some(usage.clone());
            }
        }
    }

    /// Add a single DeltaContentPart to the assembler.
    ///
    /// 添加单个 DeltaContentPart 到组装器。
    pub fn add_part(&mut self, part: DeltaContentPart) {
        match part {
            DeltaContentPart::Text {
                index,
                text,
                signature,
            } => {
                let entry = self
                    .content_parts
                    .entry(index)
                    .or_insert(AssembledPart::Text {
                        text: "".into(),
                        signature: None,
                    });
                if let AssembledPart::Text {
                    text: current_text,
                    signature: current_sig,
                } = entry
                {
                    current_text.push_str(&text);
                    if let Some(sig) = signature {
                        *current_sig = Some(sig);
                    }
                }
            }
            DeltaContentPart::Reasoning {
                index,
                text,
                signature,
            } => {
                let entry = self
                    .content_parts
                    .entry(index)
                    .or_insert(AssembledPart::Reasoning {
                        text: "".into(),
                        signature: None,
                    });
                if let AssembledPart::Reasoning {
                    text: current_text,
                    signature: current_sig,
                } = entry
                {
                    current_text.push_str(&text);
                    if let Some(sig) = signature {
                        *current_sig = Some(sig);
                    }
                }
            }
            DeltaContentPart::ToolCall(tool_call) => {
                // Determine Tool ID and cache synthetic ID in active_tool_id for performance
                let tool_id = if let Some(id) = tool_call.id {
                    self.active_tool_id.insert(tool_call.index, id.clone());
                    id
                } else {
                    self.active_tool_id
                        .entry(tool_call.index)
                        .or_insert_with(|| format!("tool_{}", tool_call.index).into())
                        .clone()
                };

                let entry = self.tool_calls.entry(tool_id.clone()).or_insert_with(|| {
                    self.tool_call_order
                        .push((tool_call.index, tool_id.clone()));
                    AssembledToolCall {
                        id: tool_id,
                        r#type: None,
                        name: None,
                        arguments: "".into(),
                        signature: None,
                    }
                });

                if let Some(tty) = tool_call.r#type.filter(|t| !t.is_empty()) {
                    entry.r#type = Some(tty);
                }
                if let Some(sig) = tool_call.signature.filter(|s| !s.is_empty()) {
                    entry.signature = Some(sig);
                }
                if let Some(func) = tool_call.function {
                    if let Some(fname) = func.name.filter(|n| !n.is_empty()) {
                        entry.name = Some(fname);
                    }
                    if let Some(fargs) = func.arguments {
                        entry.arguments.push_str(&fargs);
                    }
                }
            }
            DeltaContentPart::Refusal { .. } => {}
        }
    }

    /// Add a delta message.
    ///
    /// 添加一个增量消息。
    pub fn add(&mut self, delta: DeltaMessage) {
        self.add_metadata(&delta);
        if let Some(content) = delta.content {
            for part in content {
                self.add_part(part);
            }
        }
    }

    /// Build the complete Message.
    ///
    /// 构建完整的 Message。
    pub fn build(self) -> Message {
        let mut all_parts: Vec<(u32, ContentPart)> = Vec::new();

        // Add content parts
        for (index, part) in self.content_parts {
            let content_part = match part {
                AssembledPart::Text { text, signature } => ContentPart::Text { text, signature },
                AssembledPart::Reasoning { text, signature } => {
                    ContentPart::Reasoning { text, signature }
                }
            };
            all_parts.push((index, content_part));
        }

        // Add tool calls
        for (index, tool_id) in self.tool_call_order {
            if let Some(tool_call) = self.tool_calls.get(&tool_id) {
                all_parts.push((index, ContentPart::ToolCall(tool_call.to_tool_call())));
            }
        }

        // Sort by index. Stable sort preserves relative order for items sharing the same index.
        all_parts.sort_by_key(|(index, _)| *index);

        let content = all_parts.into_iter().map(|(_, part)| part).collect();

        Message {
            role: self.role.unwrap_or(Role::Assistant),
            content,
            name: self.name,
        }
    }

    pub fn role(&self) -> Option<Role> {
        self.role
    }

    pub fn name(&self) -> Option<StaticRefStr> {
        self.name.clone()
    }

    pub fn usage(&self) -> Option<Usage> {
        self.usage.clone()
    }

    pub fn finish_reason(&self) -> Option<FinishReason> {
        self.finish_reason.clone()
    }

    /// Get a specific tool call by index.
    ///
    /// 根据索引获取特定的工具调用。
    pub fn get_tool_call(&self, index: u32) -> Option<ToolCall> {
        // Find the LAST tool ID associated with this index in the order list.
        let tool_id = self
            .tool_call_order
            .iter()
            .rev()
            .find(|(idx, _)| *idx == index)
            .map(|(_, id)| id)?;

        self.get_tool_call_by_id(tool_id)
    }

    /// Get a specific tool call by ID.
    ///
    /// 根据 ID 获取特定的工具调用。
    pub fn get_tool_call_by_id(&self, tool_id: &str) -> Option<ToolCall> {
        let tool_call = self.tool_calls.get(tool_id)?;
        Some(tool_call.to_tool_call())
    }

    /// Get all tool call indices.
    ///
    /// 获取所有工具调用的索引。
    pub fn get_tool_call_indices(&self) -> Vec<u32> {
        self.tool_call_order.iter().map(|(idx, _)| *idx).collect()
    }
}
