use error_set::error_set;

error_set! {
    MapperError := {
        #[display("Unsupported content part for role {role} in {protocol}")]
        UnsupportedContent {
            role: String,
            protocol: String
        },
        #[display("Missing required field: {field}")]
        MissingField {
            field: String
        },
        #[display("JSON serialization error: {0}")]
        JsonError(serde_json::Error),
        #[display("Invalid media type")]
        InvalidMediaType,
        #[display("OpenAI Tool messages must correspond to exactly one ToolResult")]
        InvalidOpenAIToolMessage,
        #[display("Message with Tool role must contain ToolResult")]
        MissingToolResult,
        #[display("No choices/candidates found in response")]
        EmptyResponse,
    }
}

pub mod claude;
pub mod gemini;
pub mod openai;
