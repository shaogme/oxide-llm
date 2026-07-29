use crate::mapper::MapperError;
use crate::message::{
    ContentPart, DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, ImageSource, Message,
    Role,
};
use crate::state::{ConversationState, RawConversationState};
use crate::tool::{
    FunctionDefinition, JSONSchema, JSONSchemaType, ToolCall, ToolChoice, ToolDefinition, ToolType,
};
use oxide_llm_proto::gemini::v1beta::generate_content::{
    Blob, Content as GeminiContent, FileData, FunctionCall,
    FunctionCallingConfig as GeminiFunctionCallingConfig,
    FunctionCallingConfigMode as GeminiFunctionCallingConfigMode,
    FunctionDeclaration as GeminiFunctionDeclaration, FunctionResponse, Part as GeminiPart,
    Schema as GeminiSchema, ToolConfig as GeminiToolConfig, Type as GeminiType,
    response::GenerateContentResponse,
};
use ref_str::StaticRefStr;

/// Mapper for Gemini protocol.
///
/// Gemini 协议映射器。
pub struct GeminiGenerateContentMapper;

impl GeminiGenerateContentMapper {
    /// Convert core Message to Gemini Content.
    ///
    /// 将核心 Message 转换为 Gemini Content。
    pub fn from_core_message(msg: Message) -> Result<GeminiContent, MapperError> {
        let (role, parts) = match msg.role {
            Role::User => ("user", Self::convert_content_to_gemini_parts(msg.content)?),
            Role::Assistant => ("model", Self::convert_content_to_gemini_parts(msg.content)?),
            Role::Tool => ("user", Self::convert_content_to_gemini_parts(msg.content)?),
        };

        Ok(GeminiContent {
            parts,
            role: Some(role.into()),
        })
    }

    fn convert_content_to_gemini_parts(
        parts: Vec<ContentPart>,
    ) -> Result<Vec<GeminiPart>, MapperError> {
        let mut gemini_parts = Vec::new();
        let mut last_signature = None;

        for part in parts {
            match part {
                ContentPart::Reasoning { text, signature } => {
                    if signature.is_some() {
                        last_signature = signature.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        text: Some(text),
                        thought: Some(true),
                        thought_signature: signature,
                        ..Default::default()
                    });
                }
                ContentPart::Text { text, signature } => {
                    let sig = signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        text: Some(text),
                        thought_signature: sig,
                        ..Default::default()
                    });
                }
                ContentPart::Image(image) => {
                    // Gemini usually expects InlineData for base64
                    if let ImageSource::Base64 { data } = image.source {
                        gemini_parts.push(GeminiPart {
                            inline_data: Some(Blob {
                                mime_type: image.media_type.ok_or(MapperError::InvalidMediaType)?,
                                data,
                            }),
                            ..Default::default()
                        });
                    } else if let ImageSource::Url { url } = image.source {
                        gemini_parts.push(GeminiPart {
                            file_data: Some(FileData {
                                mime_type: image.media_type,
                                file_uri: url,
                            }),
                            ..Default::default()
                        });
                    }
                }
                ContentPart::Audio(audio) => {
                    gemini_parts.push(GeminiPart {
                        inline_data: Some(Blob {
                            mime_type: format!("audio/{}", audio.format).into(), // e.g. audio/mp3
                            data: audio.data,
                        }),
                        ..Default::default()
                    });
                }
                ContentPart::ToolCall(tc) => {
                    let sig = tc.signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        function_call: Some(FunctionCall {
                            id: Some(tc.id),
                            name: tc.name,
                            args: tc.arguments,
                        }),
                        thought_signature: sig,
                        ..Default::default()
                    });
                }
                ContentPart::ToolResult(tr) => {
                    let response_value = if tr.content.len() == 1 {
                        match &tr.content[0] {
                            ContentPart::Text { text, signature: _ } => {
                                serde_json::json!({ "content": text })
                            }
                            ContentPart::Json(value) => value.clone(),
                            _ => {
                                // Try to serialize other parts
                                serde_json::to_value(&tr.content)?
                            }
                        }
                    } else {
                        serde_json::to_value(&tr.content)?
                    };

                    let sig = tr.signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        function_response: Some(FunctionResponse {
                            id: Some(tr.tool_call_id),
                            name: tr.name,
                            response: response_value,
                            parts: None,
                            will_continue: None,
                            scheduling: None,
                        }),
                        thought_signature: sig,
                        ..Default::default()
                    });
                }
                ContentPart::Json(value) => {
                    let text = serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                    gemini_parts.push(GeminiPart {
                        text: Some(text),
                        ..Default::default()
                    });
                }
                _ => {
                    return Err(MapperError::UnsupportedContent {
                        role: "Any".to_string(),
                        protocol: "Gemini".to_string(),
                    });
                }
            }
        }
        Ok(gemini_parts)
    }

    /// Convert Gemini GenerateContentResponse to core Message.
    ///
    /// 将 Gemini GenerateContentResponse 转换为核心 Message。
    pub fn to_core_message(resp: GenerateContentResponse) -> Result<Message, MapperError> {
        let candidate = resp.candidates.first().ok_or(MapperError::EmptyResponse)?;

        let mut content_parts = Vec::new();

        let mut last_sig = None;
        for part in &candidate.content.parts {
            let sig = part.thought_signature.clone().or_else(|| last_sig.clone());
            if sig.is_some() {
                last_sig = sig.clone();
            }

            if let Some(text) = &part.text {
                if part.thought == Some(true) {
                    content_parts.push(ContentPart::Reasoning {
                        text: text.clone(),
                        signature: sig,
                    });
                } else {
                    content_parts.push(ContentPart::Text {
                        text: text.clone(),
                        signature: sig,
                    });
                }
            } else if let Some(fc) = &part.function_call {
                // Interoperability: Gemini doesn't have call_id, so use name as ID.
                content_parts.push(ContentPart::ToolCall(ToolCall {
                    id: fc.id.clone().unwrap_or_else(|| fc.name.clone()),
                    name: fc.name.clone(),
                    arguments: fc.args.clone(),
                    signature: sig,
                }));
            }
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }

    /// Convert `ToolDefinition` to `GeminiFunctionDeclaration`.
    ///
    /// 将 `ToolDefinition` 转换为 `GeminiFunctionDeclaration`。
    pub fn tool_to_gemini_function_declaration(
        tool: &ToolDefinition,
    ) -> GeminiFunctionDeclaration {
        let schema = tool
            .function
            .parameters
            .as_ref()
            .and_then(Self::json_schema_to_gemini_schema);

        GeminiFunctionDeclaration {
            name: tool.function.name.clone(),
            description: tool.function.description.clone().unwrap_or_default(),
            parameters: schema,
        }
    }

    /// Convert `ToolChoice` to `Option<GeminiToolConfig>`.
    ///
    /// 将 `ToolChoice` 转换为 `Option<GeminiToolConfig>`。
    pub fn tool_choice_to_gemini(choice: &ToolChoice) -> Option<GeminiToolConfig> {
        let (mode, allowed_function_names) = match choice {
            ToolChoice::None => (Some(GeminiFunctionCallingConfigMode::None), None),
            ToolChoice::Auto => (Some(GeminiFunctionCallingConfigMode::Auto), None),
            ToolChoice::Required => (Some(GeminiFunctionCallingConfigMode::Any), None),
            ToolChoice::Function { name } => (
                Some(GeminiFunctionCallingConfigMode::Any),
                Some(vec![name.clone()]),
            ),
        };

        Some(GeminiToolConfig {
            function_calling_config: Some(GeminiFunctionCallingConfig {
                mode,
                allowed_function_names,
            }),
            retrieval_config: None,
        })
    }

    /// Convert `GeminiFunctionDeclaration` to `ToolDefinition`.
    ///
    /// 将 `GeminiFunctionDeclaration` 转换为 `ToolDefinition`。
    pub fn tool_from_gemini(decl: GeminiFunctionDeclaration) -> ToolDefinition {
        let parameters = decl.parameters.as_ref().map(Self::gemini_schema_to_json_schema);
        ToolDefinition {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: decl.name,
                description: Some(decl.description),
                parameters,
                strict: None,
            },
        }
    }

    /// Convert `GeminiToolConfig` to `ToolChoice`.
    ///
    /// 将 `GeminiToolConfig` 转换为 `ToolChoice`。
    pub fn tool_choice_from_gemini(config: GeminiToolConfig) -> ToolChoice {
        let Some(config) = config.function_calling_config else {
            return ToolChoice::Auto;
        };

        match config.mode {
            Some(GeminiFunctionCallingConfigMode::None) => ToolChoice::None,
            Some(GeminiFunctionCallingConfigMode::Auto) => ToolChoice::Auto,
            Some(GeminiFunctionCallingConfigMode::Any) => config
                .allowed_function_names
                .filter(|names| names.len() == 1)
                .map(|names| ToolChoice::Function {
                    name: names[0].clone(),
                })
                .unwrap_or(ToolChoice::Required),
            _ => ToolChoice::Auto,
        }
    }

    /// Convert JSONSchema to Gemini Schema.
    ///
    /// 将 JSONSchema 转换为 Gemini Schema。
    pub fn json_schema_to_gemini_schema(schema: &JSONSchema) -> Option<GeminiSchema> {
        let schema_type = match schema.schema_type {
            Some(JSONSchemaType::String) => GeminiType::String,
            Some(JSONSchemaType::Number) => GeminiType::Number,
            Some(JSONSchemaType::Integer) => GeminiType::Integer,
            Some(JSONSchemaType::Boolean) => GeminiType::Boolean,
            Some(JSONSchemaType::Array) => GeminiType::Array,
            Some(JSONSchemaType::Object) => GeminiType::Object,
            Some(JSONSchemaType::Null) => GeminiType::TypeUnspecified,
            None => GeminiType::TypeUnspecified,
        };

        let properties = schema.properties.as_ref().map(|props| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in props {
                if let Some(s) = Self::json_schema_to_gemini_schema(v) {
                    map.insert(k.clone(), s);
                }
            }
            map
        });

        let items = schema
            .items
            .as_ref()
            .and_then(|v| Self::json_schema_to_gemini_schema(v))
            .map(Box::new);

        Some(GeminiSchema {
            schema_type,
            format: schema.format.clone(),
            description: schema.description.clone(),
            nullable: schema.nullable,
            r#enum: schema.enum_values.clone(),
            properties,
            required: schema.required.clone(),
            items,
        })
    }

    /// Recursively convert Gemini Schema to JSONSchema.
    ///
    /// 递归将 Gemini Schema 转换为 JSONSchema。
    pub fn gemini_schema_to_json_schema(schema: &GeminiSchema) -> JSONSchema {
        let schema_type = match schema.schema_type {
            GeminiType::String => Some(JSONSchemaType::String),
            GeminiType::Number => Some(JSONSchemaType::Number),
            GeminiType::Integer => Some(JSONSchemaType::Integer),
            GeminiType::Boolean => Some(JSONSchemaType::Boolean),
            GeminiType::Array => Some(JSONSchemaType::Array),
            GeminiType::Object => Some(JSONSchemaType::Object),
            GeminiType::TypeUnspecified => None,
        };

        let properties = schema.properties.as_ref().map(|props| {
            let mut map = std::collections::BTreeMap::new();
            for (k, v) in props {
                map.insert(k.clone(), Self::gemini_schema_to_json_schema(v));
            }
            map
        });

        let items = schema
            .items
            .as_ref()
            .map(|v| Box::new(Self::gemini_schema_to_json_schema(v)));

        JSONSchema {
            schema_type,
            description: schema.description.clone(),
            properties,
            required: schema.required.clone(),
            items,
            enum_values: schema.r#enum.clone(),
            additional_properties: None,
            format: schema.format.clone(),
            default: None,
            nullable: schema.nullable,
        }
    }
}

impl TryFrom<Message> for GeminiContent {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        GeminiGenerateContentMapper::from_core_message(msg)
    }
}

impl TryFrom<GenerateContentResponse> for Message {
    type Error = MapperError;

    fn try_from(resp: GenerateContentResponse) -> Result<Self, Self::Error> {
        GeminiGenerateContentMapper::to_core_message(resp)
    }
}

impl TryFrom<&ToolDefinition> for GeminiFunctionDeclaration {
    type Error = MapperError;

    fn try_from(tool: &ToolDefinition) -> Result<Self, Self::Error> {
        Ok(GeminiGenerateContentMapper::tool_to_gemini_function_declaration(tool))
    }
}

impl TryFrom<GeminiFunctionDeclaration> for ToolDefinition {
    type Error = MapperError;

    fn try_from(decl: GeminiFunctionDeclaration) -> Result<Self, Self::Error> {
        Ok(GeminiGenerateContentMapper::tool_from_gemini(decl))
    }
}

impl TryFrom<&ToolChoice> for GeminiToolConfig {
    type Error = MapperError;

    fn try_from(choice: &ToolChoice) -> Result<Self, Self::Error> {
        GeminiGenerateContentMapper::tool_choice_to_gemini(choice).ok_or(MapperError::MissingField {
            field: "tool_choice".to_string(),
        })
    }
}

impl TryFrom<GeminiToolConfig> for ToolChoice {
    type Error = MapperError;

    fn try_from(config: GeminiToolConfig) -> Result<Self, Self::Error> {
        Ok(GeminiGenerateContentMapper::tool_choice_from_gemini(config))
    }
}

impl TryFrom<ConversationState>
    for RawConversationState<GeminiContent, GeminiFunctionDeclaration, GeminiToolConfig>
{
    type Error = MapperError;

    fn try_from(state: ConversationState) -> Result<Self, Self::Error> {
        let messages = state
            .messages
            .into_iter()
            .map(GeminiGenerateContentMapper::from_core_message)
            .collect::<Result<Vec<_>, _>>()?;
        let tools = state
            .tools
            .iter()
            .map(GeminiGenerateContentMapper::tool_to_gemini_function_declaration)
            .collect();
        let tool_choice = state
            .tool_choice
            .as_ref()
            .and_then(GeminiGenerateContentMapper::tool_choice_to_gemini);

        Ok(RawConversationState {
            system_prompt: state.system_prompt,
            messages,
            tools,
            tool_choice,
        })
    }
}



/// A stateful mapper for Gemini streaming responses.
///
/// 用于 Gemini 流式响应的有状态映射器。
pub struct GeminiGenerateContentStreamMapper {
    last_signature: Option<StaticRefStr>,
}

impl Default for GeminiGenerateContentStreamMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiGenerateContentStreamMapper {
    pub fn new() -> Self {
        Self {
            last_signature: None,
        }
    }

    pub fn map_response(
        &mut self,
        resp: GenerateContentResponse,
    ) -> Result<DeltaMessage, MapperError> {
        let candidate = resp.candidates.first();

        // Usage info might be available at top level
        let usage = resp.usage_metadata.map(|u| crate::message::Usage {
            input_tokens: u.prompt_token_count as u32,
            output_tokens: u.candidates_token_count as u32,
            total_tokens: u.total_token_count as u32,
            reasoning_tokens: if u.thoughts_token_count > 0 {
                Some(u.thoughts_token_count as u32)
            } else {
                None
            },
            cached_input_tokens: if u.cached_content_token_count > 0 {
                Some(u.cached_content_token_count as u32)
            } else {
                None
            },
            cached_output_tokens: None,
            tool_use_tokens: None,
        });

        if candidate.is_none() {
            // Just usage update or empty
            return Ok(DeltaMessage {
                role: None,
                content: None,
                name: None,
                finish_reason: None,
                usage,
            });
        }
        let candidate = candidate.unwrap();

        let finish_reason = candidate.finish_reason.as_ref().map(|r| match r {
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Stop => {
                crate::message::FinishReason::Stop
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::MaxTokens => {
                crate::message::FinishReason::Length
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Safety => {
                crate::message::FinishReason::ContentFilter
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Recitation => {
                crate::message::FinishReason::ContentFilter
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Other => {
                crate::message::FinishReason::Other("Other".into())
            }
            _ => crate::message::FinishReason::Other(format!("{:?}", r).into()),
        });

        let mut content_parts = Vec::new();

        for (i, part) in candidate.content.parts.iter().enumerate() {
            // Update sticky signature
            if let Some(sig) = &part.thought_signature {
                self.last_signature = Some(sig.clone());
            }
            let sig = self.last_signature.clone();

            if part.thought == Some(true) {
                if let Some(text) = &part.text {
                    content_parts.push(DeltaContentPart::Reasoning {
                        index: i as u32,
                        text: text.clone(),
                        signature: sig,
                    });
                }
            } else if let Some(text) = &part.text {
                content_parts.push(DeltaContentPart::Text {
                    index: i as u32,
                    text: text.clone(),
                    signature: sig,
                });
            } else if let Some(fc) = &part.function_call {
                // Gemini sends full function call in one go usually, but for Delta we map it
                content_parts.push(DeltaContentPart::ToolCall(DeltaToolCall {
                    index: i as u32,
                    id: Some(fc.id.clone().unwrap_or_else(|| fc.name.clone())),
                    r#type: Some("function".into()),
                    function: Some(DeltaFunction {
                        name: Some(fc.name.clone()),
                        arguments: Some(
                            serde_json::to_string(&fc.args)
                                .map_err(MapperError::JsonError)?
                                .into(),
                        ),
                    }),
                    signature: sig,
                }));
            }
        }

        let role = candidate.content.role.as_deref().map(|r| match r {
            "user" => Role::User,
            "model" => Role::Assistant,
            "function" => Role::Tool,
            _ => Role::Assistant,
        });

        let content = if content_parts.is_empty() {
            None
        } else {
            Some(content_parts)
        };

        Ok(DeltaMessage {
            role,
            content,
            name: None,
            finish_reason,
            usage,
        })
    }
}
