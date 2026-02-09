use futures::StreamExt;
use oxide_llm::DynChatAgent;
use oxide_llm::agent::claude::v1::message::{MessagesAgent, MessagesRequiredConfig};
use oxide_llm::agent::gemini::v1beta::generate_content::{
    GenerateContentAgent, GenerateContentRequiredConfig,
};
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
    #[serde(rename = "gemini")]
    Gemini(GeminiConfig),
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

#[derive(Deserialize)]
struct GeminiConfig {
    base_url: String,
    endpoint: String,
    api_key: String,
    model: String,
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
        AgentConfig::Gemini(c) => {
            println!("Loaded config for Gemini model: {}", c.model);
            let transport = ReqwestTransport::new();
            let transport = AuthorizationLayer::new(transport, c.api_key);
            let transport = BaseUrlLayer::new(transport, c.base_url);

            let agent_config = GenerateContentRequiredConfig {
                model: c.model,
                endpoint: c.endpoint,
            };
            Box::new(GenerateContentAgent::new(transport, agent_config))
        }
    };

    // 4. Prepare Conversation State
    let mut state = ConversationState::new(None);

    /// Get the current weather in a given location
    #[oxide_llm::macros::tool]
    pub fn get_weather(
        /// The city name, e.g. San Francisco
        location: String,
        /// The unit of temperature
        #[tool(default = "celsius")]
        unit: String,
    ) -> String {
        format!("The weather in {} is sunny, 25 degrees {}.", location, unit)
    }

    /// Get the stock price for a given symbol
    #[oxide_llm::macros::tool]
    pub async fn get_stock_price(
        /// The stock symbol, e.g. AAPL
        symbol: String,
    ) -> Result<String, String> {
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if symbol == "AAPL" {
            Ok(format!("The stock price of {} is $150.00", symbol))
        } else {
            Err(format!("Stock symbol {} not found", symbol))
        }
    }

    let mut registry = oxide_llm::tool::ToolRegistry::new();
    registry.register(GetWeatherTool);
    registry.register(GetStockPriceTool);

    state.add_tools(registry.definitions());

    // 5. Interaction Loop
    let user_input = "What is the weather in Tokyo and what is the stock price of AAPL?";
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

        // Handle case where stream might be empty or failed immediately
        if collected_events.is_empty() {
            println!("Stream finished but no events collected.");
            // Break or Continue? If it's an error it might have been printed above.
            // If completely empty, just break to avoid loop
            break;
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

            let tool_result_bucket = if let Some(res) = registry
                .execute(&tool_call.name, tool_call.arguments.clone())
                .await
            {
                match res {
                    Ok(content) => oxide_llm::core::tool::ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        content,
                        is_error: false,
                    },
                    Err(err) => oxide_llm::core::tool::ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        content: vec![oxide_llm::core::message::ContentPart::Text {
                            text: format!("Error executing tool: {}", err),
                        }],
                        is_error: true,
                    },
                }
            } else {
                oxide_llm::core::tool::ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    content: vec![oxide_llm::core::message::ContentPart::Text {
                        text: format!("Error: Unknown tool '{}'", tool_call.name),
                    }],
                    is_error: true,
                }
            };

            // Print the result for demo purposes (extract text)
            if let Some(oxide_llm::core::message::ContentPart::Text { text }) =
                tool_result_bucket.content.first()
            {
                println!("[Tool Result] {}", text);
            }

            // Gemini specific: Tool results must be followed by the original tool call?
            // Usually the conversation state handles the history.
            // We just add tool result here.

            state.add_message(Message {
                role: oxide_llm::core::message::Role::Tool,
                content: vec![oxide_llm::core::message::ContentPart::ToolResult(
                    tool_result_bucket,
                )],
                name: None,
            });
        }
    }

    Ok(())
}
