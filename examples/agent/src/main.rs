use futures::StreamExt;
use oxide_llm::DynChatAgent;
use oxide_llm::agent::claude::v1::message::{MessagesAgent, MessagesRequiredConfig};
use oxide_llm::agent::openai::v1::chat_completions::{
    ChatCompletionsAgent, ChatCompletionsRequiredConfig,
};
use oxide_llm::core::message::{ChatStreamEvent, Message};
use oxide_llm::core::state::ConversationState;
use oxide_llm::core::transport::{AuthorizationLayer, BaseUrlLayer};
use oxide_llm_transport::reqwest::ReqwestTransport;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct Config {
    agent: AgentConfig,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum AgentConfig {
    #[serde(rename = "openai")]
    OpenAI(OpenAIConfig),
    #[serde(rename = "claude")]
    Claude(ClaudeConfig),
}

#[derive(Deserialize)]
struct OpenAIConfig {
    base_url: String,
    endpoint: String,
    api_key: String,
    model: String,
}

#[derive(Deserialize)]
struct ClaudeConfig {
    base_url: String,
    endpoint: String,
    api_key: String,
    model: String,
    max_tokens: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration
    let config_path = Path::new("examples/agent/agent.toml");
    if !config_path.exists() {
        eprintln!("Config file not found: {:?}", config_path);
        eprintln!("Please create it based on the example.");
        return Ok(());
    }

    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    // 2. Build Agent
    let agent: Box<dyn DynChatAgent> = match config.agent {
        AgentConfig::OpenAI(c) => {
            println!("Loaded config for OpenAI model: {}", c.model);
            let transport = ReqwestTransport::new();
            let transport = AuthorizationLayer::new(transport, c.api_key);
            let transport = BaseUrlLayer::new(transport, c.base_url);

            let agent_config = ChatCompletionsRequiredConfig {
                model: c.model,
                endpoint: c.endpoint,
            };
            Box::new(ChatCompletionsAgent::new(transport, agent_config))
        }
        AgentConfig::Claude(c) => {
            println!("Loaded config for Claude model: {}", c.model);
            let transport = ReqwestTransport::new();
            // Note: Claude typically uses x-api-key header, but we're reusing AuthorizationLayer for now.
            // Adjust if AuthorizationLayer is strictly Bearer token.
            let transport = AuthorizationLayer::new(transport, c.api_key);
            let transport = BaseUrlLayer::new(transport, c.base_url);

            let agent_config = MessagesRequiredConfig {
                model: c.model,
                endpoint: c.endpoint,
                max_tokens: c.max_tokens,
            };
            Box::new(MessagesAgent::new(transport, agent_config))
        }
    };

    // 4. Prepare Conversation State
    let mut state = ConversationState::new(None);

    // Define "get_weather" Tool
    let mut params = oxide_llm::core::tool::JSONSchema::object();
    let mut props = std::collections::BTreeMap::new();
    props.insert(
        "location".to_string(),
        oxide_llm::core::tool::JSONSchema::string(),
    );
    params.properties = Some(props);
    params.required = Some(vec!["location".to_string()]);

    let weather_tool = oxide_llm::core::tool::Tool::function(
        "get_weather",
        "Get the current weather in a given location",
        params,
    );

    state.add_tool(weather_tool);

    // 5. Interaction Loop
    let user_input = "What is the weather in Tokyo?";
    println!("User: {}", user_input);

    state.add_message(Message::user(user_input));

    let mut turn_count = 0;
    loop {
        if turn_count >= 5 {
            println!("Max turns reached. Exiting loop.");
            break;
        }
        turn_count += 1;

        // 6. Execute Chat (Streaming)
        println!("\n--- Agent Turn {} ---", turn_count);
        println!("Sending request...");

        // We must clone state because chat_stream takes ownership (or we could change chat_stream signature, but we are modifying only main.rs)
        let mut stream = match agent.chat_stream(state.clone()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error creating stream: {}", e);
                return Ok(());
            }
        };

        let mut is_reasoning = false;
        let mut is_text = false;
        let mut first_output = true;

        let mut collected_events = Vec::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    collected_events.push(event.clone());
                    match event {
                        ChatStreamEvent::Start { role, name } => {
                            println!("[Stream Started] Role: {:?}, Name: {:?}", role, name);
                        }
                        ChatStreamEvent::Reasoning { text } => {
                            if !is_reasoning {
                                if !first_output {
                                    println!();
                                }
                                println!("[Thinking]:");
                                is_reasoning = true;
                                is_text = false;
                                first_output = false;
                            }
                            print!("{}", text);
                            std::io::stdout().flush().unwrap();
                        }
                        ChatStreamEvent::Text { text } => {
                            if !is_text {
                                if is_reasoning {
                                    println!();
                                } else if !first_output {
                                    println!();
                                }
                                print!("Agent: ");
                                is_text = true;
                                is_reasoning = false;
                                first_output = false;
                            }
                            print!("{}", text);
                            std::io::stdout().flush().unwrap();
                        }
                        ChatStreamEvent::Finished {
                            usage,
                            finish_reason,
                        } => {
                            println!();
                            println!(
                                "[Stream Finished] Usage: {:?}, Finish Reason: {:?}",
                                usage, finish_reason
                            );
                        }
                        ChatStreamEvent::ToolCallStart { index, name, .. } => {
                            println!("\n[Tool Call Start] Index: {}, Name: {:?}", index, name);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("\nError in stream: {}", e);
                }
            }
        }

        let reconstructed_message: Message = collected_events.into_iter().collect();
        // Add the Assistant message to history
        state.add_message(reconstructed_message.clone());

        // Check for tool calls
        let tool_calls: Vec<_> = reconstructed_message
            .content
            .iter()
            .filter_map(|part| {
                if let oxide_llm::core::message::ContentPart::ToolCall(tc) = part {
                    Some(tc)
                } else {
                    None
                }
            })
            .collect();

        if tool_calls.is_empty() {
            println!("\nNo tool calls. Conversation finished.");
            break;
        }

        // Handle tool calls
        for tool_call in tool_calls {
            println!(
                "\n[Executing Tool] Name: {}, Args: {}",
                tool_call.name, tool_call.arguments
            );

            let result_content = if tool_call.name == "get_weather" {
                // Parse arguments
                let location = tool_call
                    .arguments
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                // Mock response
                format!("The weather in {} is sunny, 25 degrees Celsius.", location)
            } else {
                format!("Error: Unknown tool '{}'", tool_call.name)
            };

            println!("[Tool Result] {}", result_content);

            let tool_result = oxide_llm::core::tool::ToolResult {
                tool_call_id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                content: vec![oxide_llm::core::message::ContentPart::Text {
                    text: result_content,
                }],
                is_error: false,
            };

            state.add_message(Message {
                role: oxide_llm::core::message::Role::Tool,
                content: vec![oxide_llm::core::message::ContentPart::ToolResult(
                    tool_result,
                )],
                name: None,
            });
        }
    }

    Ok(())
}
