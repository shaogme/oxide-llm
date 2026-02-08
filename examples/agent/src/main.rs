use futures::StreamExt;
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
struct AgentConfig {
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

    println!("Loaded config for model: {}", config.agent.model);

    // 2. Build Transport Layer
    // ReqwestTransport -> AuthorizationLayer -> BaseUrlLayer
    let transport = ReqwestTransport::new();
    let transport = AuthorizationLayer::new(transport, config.agent.api_key);
    let transport = BaseUrlLayer::new(transport, config.agent.base_url);

    // 3. Initialize Agent
    let agent_config = ChatCompletionsRequiredConfig {
        model: config.agent.model,
        endpoint: config.agent.endpoint,
    };
    let agent = ChatCompletionsAgent::new(transport, agent_config);

    // 4. Prepare Conversation State
    let mut state = ConversationState::new(None);

    // 5. Interaction Loop (Single turn for now)
    let user_input = "Hello! Who are you?";
    println!("User: {}", user_input);

    state.add_message(Message::user(user_input));

    // 6. Execute Chat
    // 6. Execute Chat (Streaming)
    println!("Sending request...");
    let mut stream = match agent.chat_stream(state).await {
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
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("\nError in stream: {}", e);
            }
        }
    }

    let reconstructed_message: Message = collected_events.into_iter().collect();
    println!("\nReconstructed Message: {:#?}", reconstructed_message);

    Ok(())
}
