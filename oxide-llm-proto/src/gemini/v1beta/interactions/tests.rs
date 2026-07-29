use serde_json::json;

use super::{
    content::Content,
    request::{CreateInteractionRequest, InteractionsInput},
    response::{Interaction, InteractionStatus},
    sse::InteractionSseEvent,
    step::Step,
    tool::Tool,
};

#[test]
fn test_serialize_simple_create_interaction_request() {
    let req = CreateInteractionRequest {
        model: Some("gemini-3.6-flash".into()),
        agent: None,
        input: InteractionsInput::String("Hello, how are you?".to_string()),
        system_instruction: None,
        tools: None,
        response_format: None,
        stream: Some(false),
        store: None,
        background: None,
        generation_config: None,
        agent_config: None,
        environment: None,
        labels: None,
        previous_interaction_id: None,
        safety_settings: None,
        service_tier: None,
        webhook_config: None,
    };

    let json_val = serde_json::to_value(&req).expect("Failed to serialize request");
    assert_eq!(json_val["model"], "gemini-3.6-flash");
    assert_eq!(json_val["input"], "Hello, how are you?");
}

#[test]
fn test_deserialize_interaction_response() {
    let raw_json = json!({
        "created": "2025-11-26T12:25:15Z",
        "id": "v1_ChdPU0F4YWFtNkFwS2kxZThQZ05lbXdROBIXT1NBeGFhbTZBcEtpMWU4UGdOZW13UTg",
        "model": "gemini-3.6-flash",
        "object": "interaction",
        "status": "completed",
        "steps": [
            {
                "type": "model_output",
                "content": [
                    {
                        "type": "text",
                        "text": "Hello! I'm functioning perfectly and ready to assist you."
                    }
                ]
            }
        ],
        "updated": "2025-11-26T12:25:15Z",
        "usage": {
            "total_input_tokens": 7,
            "total_output_tokens": 20,
            "total_thought_tokens": 22,
            "total_tokens": 49
        }
    });

    let interaction: Interaction =
        serde_json::from_value(raw_json).expect("Failed to deserialize Interaction");
    assert_eq!(
        interaction.id,
        "v1_ChdPU0F4YWFtNkFwS2kxZThQZ05lbXdROBIXT1NBeGFhbTZBcEtpMWU4UGdOZW13UTg"
    );
    assert_eq!(interaction.status, InteractionStatus::Completed);
    assert_eq!(interaction.object, "interaction");

    let steps = interaction.steps.expect("Steps should exist");
    assert_eq!(steps.len(), 1);

    if let Step::ModelOutput(output) = &steps[0] {
        let contents = output.content.as_ref().expect("Content should exist");
        if let Content::Text(text_content) = &contents[0] {
            assert!(text_content
                .text
                .contains("functioning perfectly"));
        } else {
            panic!("Expected TextContent variant");
        }
    } else {
        panic!("Expected ModelOutput step variant");
    }
}

#[test]
fn test_deserialize_function_calling_interaction() {
    let raw_json = json!({
        "created": "2025-11-26T12:22:47Z",
        "id": "v1_ChdPU0F4YWFtNkFwS2kxZThQZ05lbXdROBIXT1NBeGFhbTZBcEtpMWU4UGdOZW13UTg",
        "model": "gemini-3.6-flash",
        "object": "interaction",
        "status": "requires_action",
        "steps": [
            {
                "type": "function_call",
                "id": "gth23981",
                "name": "get_weather",
                "arguments": {
                    "location": "Boston, MA"
                }
            }
        ],
        "updated": "2025-11-26T12:22:47Z"
    });

    let interaction: Interaction =
        serde_json::from_value(raw_json).expect("Failed to deserialize function call interaction");
    assert_eq!(interaction.status, InteractionStatus::RequiresAction);

    let steps = interaction.steps.expect("Steps should exist");
    if let Step::FunctionCall(func_call) = &steps[0] {
        assert_eq!(func_call.id.as_str(), "gth23981");
        assert_eq!(func_call.name.as_str(), "get_weather");
        assert_eq!(func_call.arguments["location"], "Boston, MA");
    } else {
        panic!("Expected FunctionCall step variant");
    }
}

#[test]
fn test_tool_serialization() {
    let function_tool = Tool::Function(super::tool::FunctionTool {
        name: Some("get_weather".into()),
        description: Some("Get current weather".into()),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "location": { "type": "string" }
            }
        })),
    });

    let json_val = serde_json::to_value(&function_tool).expect("Failed to serialize Tool");
    assert_eq!(json_val["type"], "function");
    assert_eq!(json_val["name"], "get_weather");
}

#[test]
fn test_sse_event_deserialization() {
    let raw_json = json!({
        "event_type": "step.delta",
        "index": 0,
        "delta": {
            "type": "text",
            "text": "Hello world"
        }
    });

    let event: InteractionSseEvent =
        serde_json::from_value(raw_json).expect("Failed to deserialize StepDelta event");
    if let InteractionSseEvent::StepDelta(step_delta) = event {
        assert_eq!(step_delta.index, 0);
        if let super::sse::StepDeltaData::Text(text_delta) = step_delta.delta {
            assert_eq!(text_delta.text.unwrap(), "Hello world");
        } else {
            panic!("Expected Text delta");
        }
    } else {
        panic!("Expected StepDelta event");
    }
}
