/// Non-stream E2E tests for `oxide-llm`.
///
/// `oxide-llm` 非流式 (Non-stream / Sync) 端到端 E2E 测试模块。
#[cfg(test)]
mod tests {
    use crate::{MockServerGuard, WeatherTool};
    use oxide_llm::{
        ChatAgent, Runner,
        agent::{
            claude::v1::message::{MessagesAgent, MessagesRequiredConfig},
            gemini::v1beta::generate_content::{
                GenerateContentAgent, GenerateContentRequiredConfig,
            },
            openai::v1::{
                chat_completions::{ChatCompletionsAgent, ChatCompletionsRequiredConfig},
                responses::{ResponsesAgent, ResponsesRequiredConfig},
            },
        },
        core::{
            message::{ContentPart, Message},
            state::ConversationState,
            transport::TransportExt,
        },
        transport::reqwest::ReqwestTransport,
    };

    #[tokio::test]
    async fn test_openai_chat_completions_non_stream_e2e() {
        let guard = MockServerGuard::start(3005);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello OpenAI Non-Stream!"));

        let res_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = res_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(text, "Hello! How can I help you today?");
    }

    #[tokio::test]
    async fn test_openai_responses_non_stream_e2e() {
        let guard = MockServerGuard::start(3015);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ResponsesRequiredConfig::new("gpt-4o", "/v1/responses");
        let agent = ResponsesAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Responses Non-Stream!"));

        let res_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = res_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(text, "Hello from OpenAI Responses Non-Stream Mock!");
    }

    #[tokio::test]
    async fn test_claude_messages_non_stream_e2e() {
        let guard = MockServerGuard::start(3006);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Claude Non-Stream!"));

        let res_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = res_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(text, "Hello from Claude Mock!");
    }

    #[tokio::test]
    async fn test_gemini_generate_content_non_stream_e2e() {
        let guard = MockServerGuard::start(3007);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            GenerateContentRequiredConfig::new("gemini-1.5-pro", "/v1beta/models/gemini-1.5-pro");
        let agent = GenerateContentAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Gemini Non-Stream!"));

        let res_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = res_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(text, "Hello from Gemini Mock!");
    }

    #[tokio::test]
    async fn test_openai_tool_call_non_stream_e2e() {
        let guard = MockServerGuard::start(3010);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Berlin?"));

        let res_msg = runner.agent().chat(state).await.expect("Chat should succeed");

        let tool_call_found = res_msg.content.iter().any(|part| match part {
            ContentPart::ToolCall(tc) => tc.name == "get_weather",
            _ => false,
        });

        assert!(
            tool_call_found,
            "Should return get_weather tool call in non-stream mode"
        );
    }

    #[tokio::test]
    async fn test_claude_tool_call_non_stream_e2e() {
        let guard = MockServerGuard::start(3011);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::new(transport, agent_config);

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in London?"));

        let res_msg = runner.agent().chat(state).await.expect("Chat should succeed");

        let tool_call_found = res_msg.content.iter().any(|part| match part {
            ContentPart::ToolCall(tc) => tc.name == "get_weather",
            _ => false,
        });

        assert!(
            tool_call_found,
            "Should return get_weather tool call for Claude in non-stream mode"
        );
    }

    #[tokio::test]
    async fn test_gemini_tool_call_non_stream_e2e() {
        let guard = MockServerGuard::start(3012);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            GenerateContentRequiredConfig::new("gemini-1.5-pro", "/v1beta/models/gemini-1.5-pro");
        let agent = GenerateContentAgent::new(transport, agent_config);

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Paris?"));

        let res_msg = runner.agent().chat(state).await.expect("Chat should succeed");

        let tool_call_found = res_msg.content.iter().any(|part| match part {
            ContentPart::ToolCall(tc) => tc.name == "get_weather",
            _ => false,
        });

        assert!(
            tool_call_found,
            "Should return get_weather tool call for Gemini in non-stream mode"
        );
    }

    #[tokio::test]
    async fn test_system_prompt_non_stream_e2e() {
        let guard = MockServerGuard::start(3013);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(Some(
            "you are a helpful assistant with system prompt".into(),
        ));
        state.add_message(Message::user("Hello"));

        let res_msg = runner.run(&mut state).await.expect("Run should succeed");

        let text = res_msg
            .content
            .iter()
            .find_map(|part| match part {
                ContentPart::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or_default();

        assert_eq!(text, "System prompt acknowledged!");
    }

    #[tokio::test]
    async fn test_http_error_handling_401_e2e() {
        let guard = MockServerGuard::start(3014);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Trigger 401 Unauthorized"));

        let result = runner.run(&mut state).await;
        assert!(
            result.is_err(),
            "401 Unauthorized response should trigger error"
        );
    }

    #[tokio::test]
    async fn test_http_error_handling_500_e2e() {
        let guard = MockServerGuard::start(3016);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Trigger 500 Internal Error"));

        let result = runner.run(&mut state).await;
        assert!(
            result.is_err(),
            "500 Internal Error response should trigger error"
        );
    }
}
