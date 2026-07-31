/// Multi-turn Tool Call Loop E2E tests for `oxide-llm`.
///
/// `oxide-llm` 多轮工具执行闭环 E2E 测试模块。
#[cfg(test)]
mod tests {
    use crate::{MockServerGuard, WeatherTool};
    use oxide_llm::{
        Runner, TransportBuilder,
        agent::openai::v1::chat_completions::{ChatCompletionsAgent, ChatCompletionsConfig},
        core::{
            message::{ContentPart, Message},
            state::ConversationState,
        },
        executor::TokioToolRegistry,
        transport::reqwest::ReqwestTransport,
    };

    #[tokio::test]
    async fn test_multi_turn_tool_execution_loop_e2e() {
        let guard = MockServerGuard::start(3020);

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            ChatCompletionsConfig::new("gpt-4o").with_endpoint("/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_raw_config(agent_config)
            .build()
            .unwrap();

        // Register WeatherTool with runner
        let registry = TokioToolRegistry::new().with_tool(WeatherTool);
        let runner = Runner::new(agent).with_registry(registry);
        let mut state = ConversationState::new();
        runner.sync_tools(&mut state);
        state.add_message(Message::user("Multi-turn weather in Beijing"));

        // runner.run will execute the tool call automatically and fetch the final answer
        let final_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = final_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(
            text, "The weather in Beijing is 20°C with sunny skies.",
            "Runner should automatically execute tool and complete multi-turn loop"
        );
    }
}
