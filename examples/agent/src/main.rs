use oxide_llm::agent::openai::v1::chat_completions::{
    ChatCompletionsAgent, ChatCompletionsRequiredConfig,
};
use oxide_llm::core::message::{ContentPart, Message};
use oxide_llm::core::state::ConversationState;
use oxide_llm::core::transport::{AuthorizationLayer, BaseUrlLayer};
use oxide_llm_transport::reqwest::ReqwestTransport;
use serde::Deserialize;
use std::fs;
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
    println!("Sending request...");
    match agent.chat(state).await {
        Ok(response_message) => {
            if let Some(ContentPart::Text { text }) = response_message.content.first() {
                println!("Agent: {}", text);
            } else {
                println!("Agent sent non-text response: {:?}", response_message);
            }
        }
        Err(e) => {
            eprintln!("Error during chat: {}", e);
        }
    }

    Ok(())
}
