use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::mapper::MapperError;
use crate::message::{
    Audio, ContentPart, DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, Document, Image,
    ImageSource, Message, Role, Usage as CoreUsage, Video,
};
use crate::state::{ConversationState, ConversationStateTrait};
use crate::tool::{FunctionDefinition, ToolCall, ToolChoice, ToolDefinition, ToolType};
use oxide_llm_proto::gemini::v1beta::interactions::{
    content::{
        AudioContent, Content, DocumentContent, ImageContent, TextContent, VideoContent,
    },
    request::{
        CreateInteractionRequest, GenerationConfig, InteractionsInput,
        ToolChoice as GeminiRequestToolChoice, Turn, TurnContent,
    },
    response::Interaction,
    sse::{InteractionSseEvent, StepDeltaData},
    step::{FunctionCallStep, FunctionResultStep, ModelOutputStep, Step, ThoughtStep, UserInputStep},
    tool::{AllowedTools, FunctionTool, Tool, ToolChoiceConfig, ToolChoiceMode},
};

/// Mapper for Gemini Interactions protocol.
///
/// Gemini Interactions 协议映射器。
pub struct GeminiInteractionsMapper;

impl GeminiInteractionsMapper {
    /// Convert core Messages, model name, tools, and tool choice to CreateInteractionRequest.
    ///
    /// 将核心 Message 列表、模型名称、工具及工具选择转换为 CreateInteractionRequest。
    pub fn from_core_messages(
        messages: Vec<Message>,
        model: Option<StaticRefStr>,
        tools: Option<Vec<ToolDefinition>>,
        tool_choice: Option<ToolChoice>,
    ) -> Result<CreateInteractionRequest, MapperError> {
        let has_advanced_steps = messages.iter().any(|msg| {
            msg.content.iter().any(|part| {
                matches!(
                    part,
                    ContentPart::ToolCall(_)
                        | ContentPart::ToolResult(_)
                        | ContentPart::Reasoning { .. }
                )
            })
        });

        let input = if has_advanced_steps {
            let mut steps = Vec::new();
            for msg in messages {
                let msg_steps = Self::from_core_message_to_steps(msg)?;
                steps.extend(msg_steps);
            }
            InteractionsInput::Steps(steps)
        } else if messages.len() == 1
            && messages[0].role == Role::User
            && messages[0].content.len() == 1
        {
            if let ContentPart::Text { ref text, .. } = messages[0].content[0] {
                InteractionsInput::String(text.clone())
            } else {
                let mut turns = Vec::new();
                for msg in messages {
                    turns.push(Self::from_core_message_to_turn(msg)?);
                }
                InteractionsInput::Turns(turns)
            }
        } else {
            let mut turns = Vec::new();
            for msg in messages {
                turns.push(Self::from_core_message_to_turn(msg)?);
            }
            InteractionsInput::Turns(turns)
        };

        let mapped_tools = tools
            .map(|ts| ts.iter().map(Self::tool_to_gemini).collect::<Vec<_>>())
            .filter(|ts| !ts.is_empty());

        let mapped_tool_choice = tool_choice.as_ref().and_then(Self::tool_choice_to_gemini);

        let generation_config = mapped_tool_choice.map(|tc| GenerationConfig {
            max_output_tokens: None,
            seed: None,
            speech_config: None,
            stop_sequences: None,
            thinking_level: None,
            thinking_summaries: None,
            tool_choice: Some(tc),
            transcription_config: None,
            video_config: None,
        });

        Ok(CreateInteractionRequest {
            model,
            agent: None,
            input,
            system_instruction: None,
            tools: mapped_tools,
            response_format: None,
            stream: None,
            store: None,
            background: None,
            generation_config,
            agent_config: None,
            environment: None,
            labels: None,
            previous_interaction_id: None,
            safety_settings: None,
            service_tier: None,
            webhook_config: None,
        })
    }

    /// Convert a single core Message into a Gemini Turn.
    ///
    /// 将单条核心 Message 转换为 Gemini Turn。
    pub fn from_core_message_to_turn(msg: Message) -> Result<Turn, MapperError> {
        let role = match msg.role {
            Role::User => "user",
            Role::Assistant => "model",
            Role::Tool => "user",
        };

        let mut contents = Vec::new();
        for part in msg.content {
            if let Some(content) = Self::convert_content_part_to_gemini_content(part)? {
                contents.push(content);
            }
        }

        let turn_content = if contents.len() == 1 {
            if let Content::Text(ref t) = contents[0] {
                TurnContent::String(t.text.clone())
            } else {
                TurnContent::Contents(contents)
            }
        } else {
            TurnContent::Contents(contents)
        };

        Ok(Turn {
            role: Some(role.into()),
            content: Some(turn_content),
        })
    }

    /// Convert a single core Message into Gemini Steps.
    ///
    /// 将单条核心 Message 转换为 Gemini Steps。
    pub fn from_core_message_to_steps(msg: Message) -> Result<Vec<Step>, MapperError> {
        let mut steps = Vec::new();

        match msg.role {
            Role::User => {
                let mut contents = Vec::new();
                for part in msg.content {
                    if let Some(content) = Self::convert_content_part_to_gemini_content(part)? {
                        contents.push(content);
                    }
                }
                steps.push(Step::UserInput(UserInputStep {
                    content: Some(contents),
                }));
            }
            Role::Assistant => {
                let mut output_contents = Vec::new();

                for part in msg.content {
                    match part {
                        ContentPart::Reasoning { text, signature } => {
                            steps.push(Step::Thought(ThoughtStep {
                                signature,
                                summary: Some(vec![Content::Text(TextContent {
                                    text,
                                    annotations: None,
                                })]),
                            }));
                        }
                        ContentPart::ToolCall(tc) => {
                            steps.push(Step::FunctionCall(FunctionCallStep {
                                id: tc.id,
                                name: tc.name,
                                arguments: tc.arguments,
                            }));
                        }
                        other => {
                            if let Some(content) =
                                Self::convert_content_part_to_gemini_content(other)?
                            {
                                output_contents.push(content);
                            }
                        }
                    }
                }

                if !output_contents.is_empty() {
                    steps.push(Step::ModelOutput(ModelOutputStep {
                        content: Some(output_contents),
                        error: None,
                    }));
                }
            }
            Role::Tool => {
                for part in msg.content {
                    if let ContentPart::ToolResult(tr) = part {
                        let result_val = if tr.content.len() == 1 {
                            match &tr.content[0] {
                                ContentPart::Text { text, .. } => {
                                    serde_json::json!({ "content": text })
                                }
                                ContentPart::Json(value) => value.clone(),
                                _ => serde_json::to_value(&tr.content)?,
                            }
                        } else {
                            serde_json::to_value(&tr.content)?
                        };

                        steps.push(Step::FunctionResult(FunctionResultStep {
                            call_id: tr.tool_call_id,
                            name: Some(tr.name),
                            result: Some(result_val),
                            is_error: Some(tr.is_error),
                        }));
                    }
                }
            }
        }

        Ok(steps)
    }

    fn convert_content_part_to_gemini_content(
        part: ContentPart,
    ) -> Result<Option<Content>, MapperError> {
        match part {
            ContentPart::Text { text, .. } => Ok(Some(Content::Text(TextContent {
                text,
                annotations: None,
            }))),
            ContentPart::Image(image) => match image.source {
                ImageSource::Base64 { data } => Ok(Some(Content::Image(ImageContent {
                    data: Some(data),
                    mime_type: image.media_type,
                    resolution: None,
                    uri: None,
                }))),
                ImageSource::Url { url } => Ok(Some(Content::Image(ImageContent {
                    data: None,
                    mime_type: image.media_type,
                    resolution: None,
                    uri: Some(url),
                }))),
            },
            ContentPart::Audio(audio) => Ok(Some(Content::Audio(AudioContent {
                data: Some(audio.data),
                mime_type: Some(format!("audio/{}", audio.format).into()),
                uri: None,
                channels: None,
                sample_rate: None,
            }))),
            ContentPart::Video(video) => match video.source {
                ImageSource::Base64 { data } => Ok(Some(Content::Video(VideoContent {
                    data: Some(data),
                    mime_type: video.media_type,
                    resolution: None,
                    uri: None,
                }))),
                ImageSource::Url { url } => Ok(Some(Content::Video(VideoContent {
                    data: None,
                    mime_type: video.media_type,
                    resolution: None,
                    uri: Some(url),
                }))),
            },
            ContentPart::Document(doc) => match doc.source {
                ImageSource::Base64 { data } => Ok(Some(Content::Document(DocumentContent {
                    data: Some(data),
                    mime_type: doc.media_type,
                    uri: None,
                }))),
                ImageSource::Url { url } => Ok(Some(Content::Document(DocumentContent {
                    data: None,
                    mime_type: doc.media_type,
                    uri: Some(url),
                }))),
            },
            ContentPart::Json(value) => {
                let text = serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                Ok(Some(Content::Text(TextContent {
                    text,
                    annotations: None,
                })))
            }
            _ => Ok(None),
        }
    }

    /// Convert Gemini Interaction response to core Message.
    ///
    /// 将 Gemini Interaction 响应转换为核心 Message。
    pub fn to_core_message(interaction: Interaction) -> Result<Message, MapperError> {
        let mut content_parts = Vec::new();

        if let Some(steps) = interaction.steps {
            for step in steps {
                match step {
                    Step::ModelOutput(output) => {
                        if let Some(contents) = output.content {
                            for content in contents {
                                match content {
                                    Content::Text(t) => {
                                        content_parts.push(ContentPart::Text {
                                            text: t.text,
                                            signature: None,
                                        });
                                    }
                                    Content::Image(img) => {
                                        if let Some(data) = img.data {
                                            content_parts.push(ContentPart::Image(Image {
                                                source: ImageSource::Base64 { data },
                                                media_type: img.mime_type,
                                                detail: None,
                                            }));
                                        } else if let Some(uri) = img.uri {
                                            content_parts.push(ContentPart::Image(Image {
                                                source: ImageSource::Url { url: uri },
                                                media_type: img.mime_type,
                                                detail: None,
                                            }));
                                        }
                                    }
                                    Content::Audio(aud) => {
                                        if let Some(data) = aud.data {
                                            let format: StaticRefStr = match &aud.mime_type {
                                                Some(m) if m.starts_with("audio/") => {
                                                    m[6..].to_string().into()
                                                }
                                                Some(m) => m.clone(),
                                                None => "mp3".into(),
                                            };
                                            content_parts.push(ContentPart::Audio(Audio {
                                                data,
                                                format,
                                            }));
                                        }
                                    }
                                    Content::Video(v) => {
                                        if let Some(data) = v.data {
                                            content_parts.push(ContentPart::Video(Video {
                                                source: ImageSource::Base64 { data },
                                                media_type: v.mime_type,
                                            }));
                                        } else if let Some(uri) = v.uri {
                                            content_parts.push(ContentPart::Video(Video {
                                                source: ImageSource::Url { url: uri },
                                                media_type: v.mime_type,
                                            }));
                                        }
                                    }
                                    Content::Document(d) => {
                                        if let Some(data) = d.data {
                                            content_parts.push(ContentPart::Document(Document {
                                                source: ImageSource::Base64 { data },
                                                media_type: d.mime_type,
                                            }));
                                        } else if let Some(uri) = d.uri {
                                            content_parts.push(ContentPart::Document(Document {
                                                source: ImageSource::Url { url: uri },
                                                media_type: d.mime_type,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Step::Thought(thought) => {
                        let text = thought
                            .summary
                            .as_ref()
                            .and_then(|s| s.first())
                            .and_then(|c| match c {
                                Content::Text(t) => Some(t.text.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();

                        content_parts.push(ContentPart::Reasoning {
                            text,
                            signature: thought.signature,
                        });
                    }
                    Step::FunctionCall(fc) => {
                        content_parts.push(ContentPart::ToolCall(ToolCall {
                            id: fc.id,
                            name: fc.name,
                            arguments: fc.arguments,
                            signature: None,
                        }));
                    }
                    Step::CodeExecutionCall(cec) => {
                        content_parts.push(ContentPart::ToolCall(ToolCall {
                            id: cec.id,
                            name: "code_execution".into(),
                            arguments: cec.arguments.unwrap_or(Value::Null),
                            signature: cec.signature,
                        }));
                    }
                    _ => {}
                }
            }
        } else if let Some(output_text) = interaction.output_text {
            content_parts.push(ContentPart::Text {
                text: output_text,
                signature: None,
            });
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }

    /// Convert ToolDefinition to Gemini Tool.
    ///
    /// 将 ToolDefinition 转换为 Gemini Tool。
    pub fn tool_to_gemini(tool: &ToolDefinition) -> Tool {
        let parameters = tool
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok());

        Tool::Function(FunctionTool {
            name: Some(tool.function.name.clone()),
            description: tool.function.description.clone(),
            parameters,
        })
    }

    /// Convert Gemini Tool to ToolDefinition.
    ///
    /// 将 Gemini Tool 转换为 ToolDefinition。
    pub fn tool_from_gemini(tool: Tool) -> Result<ToolDefinition, MapperError> {
        match tool {
            Tool::Function(ft) => {
                let name = ft.name.ok_or_else(|| MapperError::MissingField {
                    field: "tool.function.name".to_string(),
                })?;
                let parameters = match ft.parameters {
                    Some(v) => Some(serde_json::from_value(v).map_err(MapperError::JsonError)?),
                    None => None,
                };
                Ok(ToolDefinition {
                    r#type: ToolType::Function,
                    function: FunctionDefinition {
                        name,
                        description: ft.description,
                        parameters,
                        strict: None,
                    },
                })
            }
            _ => Err(MapperError::UnsupportedContent {
                role: "Tool".to_string(),
                protocol: "GeminiInteractions".to_string(),
            }),
        }
    }

    /// Convert ToolChoice to Gemini request ToolChoice.
    ///
    /// 将 ToolChoice 转换为 Gemini 请求的 ToolChoice。
    pub fn tool_choice_to_gemini(choice: &ToolChoice) -> Option<GeminiRequestToolChoice> {
        match choice {
            ToolChoice::None => Some(GeminiRequestToolChoice::Mode("none".into())),
            ToolChoice::Auto => Some(GeminiRequestToolChoice::Mode("auto".into())),
            ToolChoice::Required => Some(GeminiRequestToolChoice::Mode("any".into())),
            ToolChoice::Function { name } => {
                Some(GeminiRequestToolChoice::Config(ToolChoiceConfig {
                    allowed_tools: Some(AllowedTools {
                        mode: Some(ToolChoiceMode::Any),
                        tools: Some(vec![name.clone()]),
                    }),
                }))
            }
        }
    }

    /// Convert Gemini request ToolChoice to ToolChoice.
    ///
    /// 将 Gemini 请求的 ToolChoice 转换为 ToolChoice。
    pub fn tool_choice_from_gemini(choice: &GeminiRequestToolChoice) -> ToolChoice {
        match choice {
            GeminiRequestToolChoice::Mode(mode) => match mode.as_str() {
                "none" => ToolChoice::None,
                "any" => ToolChoice::Required,
                _ => ToolChoice::Auto,
            },
            GeminiRequestToolChoice::Config(config) => {
                if let Some(allowed) = &config.allowed_tools {
                    if let Some(tools) = &allowed.tools
                        && tools.len() == 1
                    {
                        return ToolChoice::Function {
                            name: tools[0].clone(),
                        };
                    }
                    if matches!(allowed.mode, Some(ToolChoiceMode::Any)) {
                        return ToolChoice::Required;
                    }
                    if matches!(allowed.mode, Some(ToolChoiceMode::None)) {
                        return ToolChoice::None;
                    }
                }
                ToolChoice::Auto
            }
        }
    }
}

impl TryFrom<Interaction> for Message {
    type Error = MapperError;

    fn try_from(interaction: Interaction) -> Result<Self, Self::Error> {
        GeminiInteractionsMapper::to_core_message(interaction)
    }
}

impl TryFrom<&ToolDefinition> for Tool {
    type Error = MapperError;

    fn try_from(tool: &ToolDefinition) -> Result<Self, Self::Error> {
        Ok(GeminiInteractionsMapper::tool_to_gemini(tool))
    }
}

impl TryFrom<Tool> for ToolDefinition {
    type Error = MapperError;

    fn try_from(tool: Tool) -> Result<Self, Self::Error> {
        GeminiInteractionsMapper::tool_from_gemini(tool)
    }
}

impl TryFrom<&ToolChoice> for GeminiRequestToolChoice {
    type Error = MapperError;

    fn try_from(choice: &ToolChoice) -> Result<Self, Self::Error> {
        GeminiInteractionsMapper::tool_choice_to_gemini(choice).ok_or(MapperError::MissingField {
            field: "tool_choice".to_string(),
        })
    }
}

impl TryFrom<&GeminiRequestToolChoice> for ToolChoice {
    type Error = MapperError;

    fn try_from(choice: &GeminiRequestToolChoice) -> Result<Self, Self::Error> {
        Ok(GeminiInteractionsMapper::tool_choice_from_gemini(choice))
    }
}

impl TryFrom<Message> for Turn {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        GeminiInteractionsMapper::from_core_message_to_turn(msg)
    }
}

/// Gemini Interactions Raw Conversation State.
///
/// Gemini Interactions 原始对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionsConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<StaticRefStr>,

    /// Raw Message list.
    ///
    /// 原始消息列表。
    pub messages: Vec<Step>,

    /// Available raw tools.
    ///
    /// 可用原始工具列表。
    pub tools: Vec<Tool>,

    /// Raw Tool choice preference.
    ///
    /// 原始工具选择偏好。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<GeminiRequestToolChoice>,
}

impl ConversationStateTrait for InteractionsConversationState {}

impl TryFrom<ConversationState> for InteractionsConversationState {
    type Error = MapperError;

    fn try_from(state: ConversationState) -> Result<Self, Self::Error> {
        let mut steps = Vec::new();
        for msg in state.messages {
            let msg_steps = GeminiInteractionsMapper::from_core_message_to_steps(msg)?;
            steps.extend(msg_steps);
        }
        let tools = state
            .tools
            .iter()
            .map(GeminiInteractionsMapper::tool_to_gemini)
            .collect();
        let tool_choice = state
            .tool_choice
            .as_ref()
            .and_then(GeminiInteractionsMapper::tool_choice_to_gemini);

        Ok(InteractionsConversationState {
            system_prompt: state.system_prompt,
            messages: steps,
            tools,
            tool_choice,
        })
    }
}



/// A stateful mapper for Gemini Interactions streaming SSE responses.
///
/// 用于 Gemini Interactions 流式 SSE 响应的有状态映射器。
pub struct GeminiInteractionsStreamMapper {
    last_signature: Option<StaticRefStr>,
}

impl Default for GeminiInteractionsStreamMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiInteractionsStreamMapper {
    /// Create a new GeminiInteractionsStreamMapper.
    ///
    /// 创建一个新的 GeminiInteractionsStreamMapper。
    pub fn new() -> Self {
        Self {
            last_signature: None,
        }
    }

    /// Map a Gemini InteractionSseEvent to a core DeltaMessage.
    ///
    /// 将 Gemini InteractionSseEvent 映射为核心 DeltaMessage。
    pub fn map_event(&mut self, event: InteractionSseEvent) -> Result<DeltaMessage, MapperError> {
        match event {
            InteractionSseEvent::StepDelta(step_delta) => {
                let mut content_parts = Vec::new();
                let index = step_delta.index as u32;

                match step_delta.delta {
                    StepDeltaData::Text(text_delta) => {
                        if let Some(text) = text_delta.text {
                            content_parts.push(DeltaContentPart::Text {
                                index,
                                text,
                                signature: self.last_signature.clone(),
                            });
                        }
                    }
                    StepDeltaData::ThoughtSummary(thought_delta) => {
                        if let Some(text) = thought_delta.text {
                            content_parts.push(DeltaContentPart::Reasoning {
                                index,
                                text,
                                signature: self.last_signature.clone(),
                            });
                        }
                    }
                    StepDeltaData::ThoughtSignature(sig_delta) => {
                        if let Some(sig) = sig_delta.signature {
                            self.last_signature = Some(sig);
                        }
                    }
                    StepDeltaData::Arguments(args_delta) => {
                        if let Some(args) = args_delta.arguments {
                            content_parts.push(DeltaContentPart::ToolCall(DeltaToolCall {
                                index,
                                id: None,
                                r#type: Some("function".into()),
                                function: Some(DeltaFunction {
                                    name: None,
                                    arguments: Some(args.into()),
                                }),
                                signature: self.last_signature.clone(),
                            }));
                        }
                    }
                    StepDeltaData::Audio(audio_delta) => {
                        content_parts.push(DeltaContentPart::Audio {
                            index,
                            data: audio_delta.data,
                            mime_type: audio_delta.mime_type,
                        });
                    }
                    StepDeltaData::Image(image_delta) => {
                        content_parts.push(DeltaContentPart::Image {
                            index,
                            data: image_delta.data,
                            mime_type: image_delta.mime_type,
                            uri: image_delta.uri,
                        });
                    }
                    StepDeltaData::Video(video_delta) => {
                        content_parts.push(DeltaContentPart::Video {
                            index,
                            data: video_delta.data,
                            mime_type: video_delta.mime_type,
                            uri: video_delta.uri,
                        });
                    }
                    StepDeltaData::Document(doc_delta) => {
                        content_parts.push(DeltaContentPart::Document {
                            index,
                            data: doc_delta.data,
                            mime_type: doc_delta.mime_type,
                            uri: doc_delta.uri,
                        });
                    }
                }

                let content = if content_parts.is_empty() {
                    None
                } else {
                    Some(content_parts)
                };

                Ok(DeltaMessage {
                    role: Some(Role::Assistant),
                    content,
                    name: None,
                    finish_reason: None,
                    usage: None,
                })
            }
            InteractionSseEvent::StepStop(step_stop) => {
                let usage = step_stop
                    .usage
                    .or(step_stop.step_usage)
                    .map(Self::map_gemini_usage);

                Ok(DeltaMessage {
                    role: None,
                    content: None,
                    name: None,
                    finish_reason: None,
                    usage,
                })
            }
            InteractionSseEvent::InteractionCompleted(comp) => {
                let usage = comp.interaction.usage.map(Self::map_gemini_usage);

                Ok(DeltaMessage {
                    role: None,
                    content: None,
                    name: None,
                    finish_reason: Some(crate::message::FinishReason::Stop),
                    usage,
                })
            }
            InteractionSseEvent::StepStart(step_start) => {
                let index = step_start.index as u32;
                if let Step::FunctionCall(fc) = step_start.step {
                    let part = DeltaContentPart::ToolCall(DeltaToolCall {
                        index,
                        id: Some(fc.id),
                        r#type: Some("function".into()),
                        function: Some(DeltaFunction {
                            name: Some(fc.name),
                            arguments: if fc.arguments.is_null() {
                                None
                            } else {
                                serde_json::to_string(&fc.arguments).ok().map(Into::into)
                            },
                        }),
                        signature: None,
                    });
                    Ok(DeltaMessage {
                        role: Some(Role::Assistant),
                        content: Some(vec![part]),
                        name: None,
                        finish_reason: None,
                        usage: None,
                    })
                } else {
                    Ok(DeltaMessage::default())
                }
            }
            InteractionSseEvent::InteractionCreated(_)
            | InteractionSseEvent::InteractionStatusUpdate(_)
            | InteractionSseEvent::Error(_) => Ok(DeltaMessage::default()),
        }
    }

    fn map_gemini_usage(u: oxide_llm_proto::gemini::v1beta::interactions::response::Usage) -> CoreUsage {
        let reasoning_tokens = if u.total_thought_tokens > 0 {
            Some(u.total_thought_tokens as u32)
        } else {
            None
        };

        let cached_input_tokens = if u.total_cached_tokens > 0 {
            Some(u.total_cached_tokens as u32)
        } else {
            None
        };

        let tool_use_tokens = if u.total_tool_use_tokens > 0 {
            Some(u.total_tool_use_tokens as u32)
        } else {
            None
        };

        CoreUsage {
            input_tokens: u.total_input_tokens as u32,
            output_tokens: u.total_output_tokens as u32,
            total_tokens: u.total_tokens as u32,
            reasoning_tokens,
            cached_input_tokens,
            cached_output_tokens: None,
            tool_use_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::JSONSchema;
    use oxide_llm_proto::gemini::v1beta::interactions::response::InteractionStatus;
    use serde_json::json;

    #[test]
    fn test_from_core_messages_simple_turn() {
        let msg = Message {
            role: Role::User,
            content: vec![ContentPart::Text {
                text: "Hello, Gemini!".into(),
                signature: None,
            }],
            name: None,
        };

        let req = GeminiInteractionsMapper::from_core_messages(
            vec![msg],
            Some("gemini-3.6-flash".into()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(req.model.as_deref(), Some("gemini-3.6-flash"));
        if let InteractionsInput::String(text) = req.input {
            assert_eq!(text.as_str(), "Hello, Gemini!");
        } else {
            panic!("Expected String input");
        }
    }

    #[test]
    fn test_from_core_messages_with_tool_call_steps() {
        let user_msg = Message {
            role: Role::User,
            content: vec![ContentPart::Text {
                text: "What is the weather?".into(),
                signature: None,
            }],
            name: None,
        };

        let assistant_msg = Message {
            role: Role::Assistant,
            content: vec![ContentPart::ToolCall(ToolCall {
                id: "call_123".into(),
                name: "get_weather".into(),
                arguments: json!({"location": "Beijing"}),
                signature: None,
            })],
            name: None,
        };

        let req = GeminiInteractionsMapper::from_core_messages(
            vec![user_msg, assistant_msg],
            Some("gemini-3.6-flash".into()),
            None,
            None,
        )
        .unwrap();

        if let InteractionsInput::Steps(steps) = req.input {
            assert_eq!(steps.len(), 2);
            if let Step::UserInput(input) = &steps[0] {
                assert!(input.content.is_some());
            } else {
                panic!("Expected UserInput step");
            }
            if let Step::FunctionCall(fc) = &steps[1] {
                assert_eq!(fc.id.as_str(), "call_123");
                assert_eq!(fc.name.as_str(), "get_weather");
            } else {
                panic!("Expected FunctionCall step");
            }
        } else {
            panic!("Expected Steps input");
        }
    }

    #[test]
    fn test_to_core_message_from_interaction() {
        let interaction = Interaction {
            id: "v1_test123".to_string(),
            object: "interaction".to_string(),
            status: InteractionStatus::Completed,
            model: Some("gemini-3.6-flash".into()),
            agent: None,
            steps: Some(vec![
                Step::Thought(ThoughtStep {
                    signature: Some("sig_abc".into()),
                    summary: Some(vec![Content::Text(TextContent {
                        text: "Thinking process".to_string(),
                        annotations: None,
                    })]),
                }),
                Step::ModelOutput(ModelOutputStep {
                    content: Some(vec![Content::Text(TextContent {
                        text: "Hello world!".to_string(),
                        annotations: None,
                    })]),
                    error: None,
                }),
            ]),
            usage: None,
            output_text: None,
            output_image: None,
            output_audio: None,
            output_video: None,
            environment_id: None,
            created: None,
            updated: None,
            system_instruction: None,
            tools: None,
            generation_config: None,
            agent_config: None,
            environment: None,
            labels: None,
            previous_interaction_id: None,
            response_format: None,
            response_modalities: None,
            safety_settings: None,
            service_tier: None,
            webhook_config: None,
        };

        let msg = GeminiInteractionsMapper::to_core_message(interaction).unwrap();
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content.len(), 2);

        if let ContentPart::Reasoning { text, signature } = &msg.content[0] {
            assert_eq!(text, "Thinking process");
            assert_eq!(signature.as_deref(), Some("sig_abc"));
        } else {
            panic!("Expected Reasoning content part");
        }

        if let ContentPart::Text { text, .. } = &msg.content[1] {
            assert_eq!(text, "Hello world!");
        } else {
            panic!("Expected Text content part");
        }
    }

    #[test]
    fn test_tool_conversion_bidirectional() {
        let tool_def = ToolDefinition::function("get_weather", "Get current weather", JSONSchema::object());
        let gemini_tool = GeminiInteractionsMapper::tool_to_gemini(&tool_def);

        if let Tool::Function(ft) = &gemini_tool {
            assert_eq!(ft.name.as_deref(), Some("get_weather"));
            assert_eq!(ft.description.as_deref(), Some("Get current weather"));
        } else {
            panic!("Expected Function Tool");
        }

        let back_tool = GeminiInteractionsMapper::tool_from_gemini(gemini_tool).unwrap();
        assert_eq!(back_tool.function.name, "get_weather");
    }

    #[test]
    fn test_multimodal_video_and_document_mapping() {
        let msg = Message {
            role: Role::User,
            content: vec![
                ContentPart::Video(Video {
                    source: ImageSource::Url { url: "https://example.com/video.mp4".into() },
                    media_type: Some("video/mp4".into()),
                }),
                ContentPart::Document(Document {
                    source: ImageSource::Url { url: "https://example.com/doc.pdf".into() },
                    media_type: Some("application/pdf".into()),
                }),
            ],
            name: None,
        };

        let req = GeminiInteractionsMapper::from_core_messages(
            vec![msg],
            Some("gemini-3.6-flash".into()),
            None,
            None,
        )
        .unwrap();

        if let InteractionsInput::Turns(turns) = req.input {
            assert_eq!(turns.len(), 1);
            if let Some(TurnContent::Contents(contents)) = &turns[0].content {
                assert_eq!(contents.len(), 2);
                assert!(matches!(&contents[0], Content::Video(_)));
                assert!(matches!(&contents[1], Content::Document(_)));
            } else {
                panic!("Expected TurnContent::Contents");
            }
        } else {
            panic!("Expected Turns input");
        }
    }

    #[test]
    fn test_stream_mapper_map_event() {
        let mut mapper = GeminiInteractionsStreamMapper::new();

        let raw_json = json!({
            "event_type": "step.delta",
            "index": 0,
            "delta": {
                "type": "text",
                "text": "Hello streaming"
            }
        });

        let event: InteractionSseEvent = serde_json::from_value(raw_json).unwrap();
        let delta = mapper.map_event(event).unwrap();

        assert_eq!(delta.role, Some(Role::Assistant));
        let content = delta.content.expect("Content should exist");
        if let DeltaContentPart::Text { text, .. } = &content[0] {
            assert_eq!(text, "Hello streaming");
        } else {
            panic!("Expected Text delta part");
        }
    }

    #[test]
    fn test_stream_mapper_multimodal_and_usage_event() {
        let mut mapper = GeminiInteractionsStreamMapper::new();

        let audio_json = json!({
            "event_type": "step.delta",
            "index": 0,
            "delta": {
                "type": "audio",
                "data": "bWFwYXVkaW8=",
                "mime_type": "audio/mp3"
            }
        });
        let event: InteractionSseEvent = serde_json::from_value(audio_json).unwrap();
        let delta = mapper.map_event(event).unwrap();
        let parts = delta.content.unwrap();
        if let DeltaContentPart::Audio { data, mime_type, .. } = &parts[0] {
            assert_eq!(data.as_deref(), Some("bWFwYXVkaW8="));
            assert_eq!(mime_type.as_deref(), Some("audio/mp3"));
        } else {
            panic!("Expected Audio delta");
        }

        let stop_json = json!({
            "event_type": "step.stop",
            "index": 0,
            "usage": {
                "total_input_tokens": 100,
                "total_output_tokens": 50,
                "total_thought_tokens": 20,
                "total_cached_tokens": 10,
                "total_tool_use_tokens": 5,
                "total_tokens": 150
            }
        });
        let event: InteractionSseEvent = serde_json::from_value(stop_json).unwrap();
        let delta = mapper.map_event(event).unwrap();
        let usage = delta.usage.expect("Usage should exist");
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.reasoning_tokens, Some(20));
        assert_eq!(usage.cached_input_tokens, Some(10));
        assert_eq!(usage.tool_use_tokens, Some(5));
    }
}
