use clap::Parser;
use futures::StreamExt;
use oxide_llm::{
    DynChatAgent, Runner,
    agent::{
        claude::v1::message::{MessagesAgent, MessagesRequiredConfig},
        gemini::v1beta::generate_content::{GenerateContentAgent, GenerateContentRequiredConfig},
        openai::v1::{
            chat_completions::{ChatCompletionsAgent, ChatCompletionsRequiredConfig},
            responses::{ResponsesAgent, ResponsesRequiredConfig},
        },
    },
    core::{
        message::{ChatStreamEvent, Message},
        state::ConversationState,
        tool::Tool,
        transport::TransportExt,
    },
    macros::Schema,
};
use oxide_llm_transport::reqwest::ReqwestTransport;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs,
    future::Future,
    io::{Write, stdout},
    path::{Path, PathBuf},
    pin::Pin,
    time::Duration,
};
use tokio::time::sleep;

#[derive(Deserialize, Clone)]
struct SecretString(String);

impl Debug for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "********")
    }
}

impl Display for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "********")
    }
}

impl SecretString {
    fn expose(&self) -> &str {
        &self.0
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the agent to use
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    agents: Vec<NamedAgentConfig>,
}

#[derive(Deserialize)]
struct NamedAgentConfig {
    name: String,
    #[serde(flatten)]
    config: AgentConfig,
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
    api_key: SecretString,
    model: String,
}

#[derive(Deserialize)]
struct ClaudeConfig {
    base_url: String,
    endpoint: String,
    api_key: SecretString,
    model: String,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiConfig {
    base_url: String,
    endpoint: String,
    api_key: SecretString,
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
    type Error = String;
    type Future = std::future::Ready<Result<Self::Output, Self::Error>>;

    fn run(&self, args: Self::Args) -> Self::Future {
        // Simulate using state (api_key)
        if self.api_key.is_empty() {
            return std::future::ready(Err("Missing API Key for Weather Service".to_string()));
        }

        let unit = args.unit.unwrap_or_else(|| "celsius".to_string());

        // Mock response
        std::future::ready(Ok(WeatherOutput {
            location: args.location,
            temperature: 25,
            unit,
            condition: "sunny".to_string(),
        }))
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
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;

    fn run(&self, args: Self::Args) -> Self::Future {
        Box::pin(async move {
            let symbol = args.symbol.to_uppercase();

            // Simulate network delay
            sleep(Duration::from_millis(500)).await;

            if symbol == "AAPL" {
                Ok(StockOutput {
                    symbol,
                    price: 150.00,
                    currency: "USD".to_string(),
                })
            } else {
                Err(format!("Stock symbol {} not found", symbol))
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_args = Args::parse();

    // 1. Load configuration
    let config_path = if Path::new("agent.toml").exists() {
        PathBuf::from("agent.toml")
    } else {
        PathBuf::from("examples/agent/agent.toml")
    };

    if !config_path.exists() {
        eprintln!("Config file not found in current directory or 'examples/agent/'");
        return Ok(());
    }

    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    // 2. Select Agent
    let named_config = if let Some(name) = &cli_args.name {
        config
            .agents
            .iter()
            .find(|a| &a.name == name)
            .ok_or_else(|| format!("Agent with name '{}' not found", name))?
    } else {
        config
            .agents
            .first()
            .ok_or_else(|| "No agents configured".to_string())?
    };

    println!("Using agent: {}", named_config.name);

    // 3. Build Agent
    let agent: Box<dyn DynChatAgent> = match &named_config.config {
        AgentConfig::OpenAI(c) => {
            println!("Loaded config for OpenAI model: {}", c.model);
            let transport = ReqwestTransport::new()
                .with_authorization(c.api_key.expose().to_string())
                .with_base_url(c.base_url.clone());

            if c.endpoint.contains("responses") {
                let agent_config =
                    ResponsesRequiredConfig::new(c.model.clone(), c.endpoint.clone());
                Box::new(ResponsesAgent::new(transport, agent_config))
            } else {
                let agent_config =
                    ChatCompletionsRequiredConfig::new(c.model.clone(), c.endpoint.clone());
                Box::new(ChatCompletionsAgent::new(transport, agent_config))
            }
        }
        AgentConfig::Claude(c) => {
            println!("Loaded config for Claude model: {}", c.model);
            let transport = ReqwestTransport::new()
                .with_authorization(c.api_key.expose().to_string())
                .with_base_url(c.base_url.clone());

            let agent_config =
                MessagesRequiredConfig::new(c.model.clone(), c.max_tokens, c.endpoint.clone());
            Box::new(MessagesAgent::new(transport, agent_config))
        }
        AgentConfig::Gemini(c) => {
            println!("Loaded config for Gemini model: {}", c.model);
            let transport = ReqwestTransport::new()
                .with_authorization(c.api_key.expose().to_string())
                .with_base_url(c.base_url.clone());

            let agent_config =
                GenerateContentRequiredConfig::new(c.model.clone(), c.endpoint.clone());
            Box::new(GenerateContentAgent::new(transport, agent_config))
        }
    };

    // 4. Create Runner and Register Tools
    let weather_tool = WeatherTool {
        api_key: "dummy_weather_api_key".to_string(),
    };
    let stock_tool = StockTool;

    let runner = Runner::new(agent)
        .with_tool(weather_tool)
        .with_tool(stock_tool)
        .with_max_turns(5);

    // 5. Prepare Conversation State
    let mut state = ConversationState::new(None);
    runner.sync_tools(&mut state);

    // 6. Interaction Loop
    let user_input = "What is the weather in Tokyo and what is the stock price of AAPL?";
    println!("User: {}", user_input);

    state.add_message(Message::user(user_input));

    let mut stream = runner.run_stream(&mut state);

    let mut in_reasoning = false;
    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                match &event {
                    ChatStreamEvent::Reasoning { .. } => {
                        if !in_reasoning {
                            print!("[Thinking] ");
                            in_reasoning = true;
                        }
                    }
                    _ => {
                        if in_reasoning {
                            println!();
                            in_reasoning = false;
                        }
                    }
                }

                match event {
                    ChatStreamEvent::Start { role, name } => {
                        println!("[Stream Started] Role: {:?}, Name: {:?}", role, name);
                    }
                    ChatStreamEvent::Reasoning { text } => {
                        print!("{}", text);
                        stdout().flush().unwrap();
                    }
                    ChatStreamEvent::Text { text } => {
                        print!("{}", text);
                        stdout().flush().unwrap();
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
                    ChatStreamEvent::ToolCallFinished(tc) => {
                        println!("[Tool Call Finished] Id: {}, Name: {}, Args: {}", tc.id, tc.name, tc.arguments);
                    }
                }
            }
            Err(e) => {
                eprintln!("\nError in stream: {}", e);
            }
        }
    }

    Ok(())
}
