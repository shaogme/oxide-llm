/// Stream E2E tests for `oxide-llm`.
///
/// `oxide-llm` 流式 (Stream) 端到端 E2E 测试模块。
#[cfg(test)]
mod tests {
    use crate::{MockServerGuard, WeatherTool};
    use futures::StreamExt;
    use oxide_llm::{
        Runner,
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
            message::{ChatStreamEvent, Message},
            state::ConversationState,
            transport::TransportExt,
        },
        transport::reqwest::ReqwestTransport,
    };

    #[tokio::test]
    async fn test_openai_chat_completions_stream_e2e() {
        let guard = MockServerGuard::start(3001);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello OpenAI!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert!(
            !received_text.is_empty(),
            "Should receive streamed text from mock OpenAI server"
        );
    }

    #[tokio::test]
    async fn test_openai_responses_stream_e2e() {
        let guard = MockServerGuard::start(3017);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ResponsesRequiredConfig::new("gpt-4o", "/v1/responses");
        let agent = ResponsesAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Responses Stream!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert_eq!(received_text, "Hello from OpenAI Responses Stream Mock!");
    }

    #[tokio::test]
    async fn test_openai_tool_call_stream_e2e() {
        let guard = MockServerGuard::start(3004);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::new(transport, agent_config);

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Tokyo?"));

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event {
                if tc.name == "get_weather" {
                    tool_called = true;
                }
            }
        }

        assert!(
            tool_called,
            "Tool 'get_weather' should be triggered during E2E run"
        );
    }

    #[tokio::test]
    async fn test_claude_messages_stream_e2e() {
        let guard = MockServerGuard::start(3002);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Claude!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert_eq!(received_text, "Hello from Claude Mock!");
    }

    #[tokio::test]
    async fn test_claude_tool_call_stream_e2e() {
        let guard = MockServerGuard::start(3018);

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

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event {
                if tc.name == "get_weather" {
                    tool_called = true;
                }
            }
        }

        assert!(
            tool_called,
            "Tool 'get_weather' should be triggered for Claude stream"
        );
    }

    #[tokio::test]
    async fn test_gemini_generate_content_stream_e2e() {
        let guard = MockServerGuard::start(3003);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            GenerateContentRequiredConfig::new("gemini-1.5-pro", "/v1beta/models/gemini-1.5-pro");
        let agent = GenerateContentAgent::new(transport, agent_config);

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Gemini!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert_eq!(received_text, "Hello from Gemini Stream Mock!");
    }

    #[tokio::test]
    async fn test_gemini_tool_call_stream_e2e() {
        let guard = MockServerGuard::start(3019);

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

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event {
                if tc.name == "get_weather" {
                    tool_called = true;
                }
            }
        }

        assert!(
            tool_called,
            "Tool 'get_weather' should be triggered for Gemini stream"
        );
    }
}
