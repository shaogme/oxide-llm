use diagweave::set;

set! {
    pub MapperError = {
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
        JsonError(#[from] serde_json::Error),
        #[display("Invalid media type")]
        InvalidMediaType,
        #[display("OpenAI Tool messages must correspond to exactly one ToolResult")]
        InvalidOpenAIToolMessage,
        #[display("ToolResult content part found in non-Tool role message")]
        InvalidToolResultLocation,
        #[display("Tool role message contains non-ToolResult content part")]
        UnexpectedContentInToolMessage,
        #[display("No choices/candidates found in response")]
        EmptyResponse,
        #[display("Ignored stream event: {event_type}")]
        IgnoredEvent {
            event_type: String,
        },
    }
}

pub mod claude;
pub mod gemini;
pub mod openai;
