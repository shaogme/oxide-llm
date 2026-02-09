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
use oxide_llm::core::tool::Tool;
use oxide_llm::core::transport::{AuthorizationLayer, BaseUrlLayer};
use oxide_llm::macros::Schema;
use oxide_llm_transport::reqwest::ReqwestTransport;
use serde::{Deserialize, Serialize};
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

// --- Tool Definitions ---

// 1. Weather Tool

/// Arguments for the weather tool.
///
/// Get the current weather in a given location
#[derive(Deserialize, Schema)]
pub struct WeatherArgs {
    /// The city name, e.g. San Francisco
    pub location: String,
    /// The unit of temperature (celsius/fahrenheit)
    pub unit: Option<String>,
}

#[derive(Serialize)]
pub struct WeatherOutput {
    pub location: String,
    pub temperature: i32,
    pub unit: String,
    pub condition: String,
}

#[derive(Clone)]
pub struct WeatherTool {
    // Example state
    pub api_key: String,
}

impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";
    const DESCRIPTION: &'static str = "Get the current weather in a given location";

    type Args = WeatherArgs;
    type Output = WeatherOutput;

    async fn run(&self, args: Self::Args) -> Result<Self::Output, String> {
        // Simulate using state (api_key)
        if self.api_key.is_empty() {
            return Err("Missing API Key for Weather Service".to_string());
        }

        let unit = args.unit.unwrap_or_else(|| "celsius".to_string());

        // Mock response
        Ok(WeatherOutput {
            location: args.location,
            temperature: 25,
            unit,
            condition: "sunny".to_string(),
        })
    }
}

// 2. Stock Price Tool

/// Arguments for the stock price tool.
///
/// Get the stock price for a given symbol
#[derive(Deserialize, Schema)]
pub struct StockArgs {
    /// The stock symbol, e.g. AAPL
    pub symbol: String,
}

#[derive(Serialize)]
pub struct StockOutput {
    pub symbol: String,
    pub price: f64,
    pub currency: String,
}

#[derive(Clone)]
pub struct StockTool;

impl Tool for StockTool {
    const NAME: &'static str = "get_stock_price";
    const DESCRIPTION: &'static str = "Get the stock price for a given symbol";

    type Args = StockArgs;
    type Output = StockOutput;

    async fn run(&self, args: Self::Args) -> Result<Self::Output, String> {
        let symbol = args.symbol.to_uppercase();

        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if symbol == "AAPL" {
            Ok(StockOutput {
                symbol,
                price: 150.00,
                currency: "USD".to_string(),
            })
        } else {
            Err(format!("Stock symbol {} not found", symbol))
        }
    }
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

    // 5. Register Tools (Stateful)
    let mut registry = oxide_llm::tool::ToolRegistry::new();

    // Instantiate tools with state
    let weather_tool = WeatherTool {
        api_key: "dummy_weather_api_key".to_string(),
    };
    let stock_tool = StockTool;

    registry.register(weather_tool);
    registry.register(stock_tool);

    state.add_tools(registry.definitions());

    // 6. Interaction Loop
    let user_input = "What is the weather in Tokyo and what is the stock price of AAPL?";
    println!("User: {}", user_input);

    state.add_message(Message::user(user_input));

    let mut runner = oxide_llm::runner::RunnerStream::new(&*agent, &registry, &mut state, 5);

    while let Some(event_result) = runner.next().await {
        match event_result {
            Ok(event) => match event {
                ChatStreamEvent::Start { role, name } => {
                    println!("[Stream Started] Role: {:?}, Name: {:?}", role, name);
                }
                ChatStreamEvent::Reasoning { text } => {
                    print!("[Thinking] {}", text);
                    std::io::stdout().flush().unwrap();
                }
                ChatStreamEvent::Text { text } => {
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
            },
            Err(e) => {
                eprintln!("\nError in stream: {}", e);
            }
        }
    }

    Ok(())
}
