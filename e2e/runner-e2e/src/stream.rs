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
            gemini::v1beta::{
                generate_content::{GenerateContentAgent, GenerateContentRequiredConfig},
                interactions::{InteractionsAgent, InteractionsRequiredConfig},
            },
            openai::v1::{
                chat_completions::{ChatCompletionsAgent, ChatCompletionsRequiredConfig},
                responses::{ResponsesAgent, ResponsesConfig, ResponsesRequiredConfig},
            },
        },
        core::{
            message::{ChatStreamEvent, Message},
            state::ConversationState,
            transport::TransportExt,
        },
        proto::openai::v1::response::ResponseStreamEvent,
        transport::reqwest::ReqwestTransport,
    };

    #[tokio::test]
    async fn test_openai_chat_completions_stream_e2e() {
        let guard = MockServerGuard::start(3001);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

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
        let agent = ResponsesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

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
    async fn test_openai_responses_stateful_stream_e2e() {
        use oxide_llm::ChatStreamConfig;
        use std::sync::{Arc, Mutex};

        let guard = MockServerGuard::start(3026);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let required = ResponsesRequiredConfig::new("gpt-4o", "/v1/responses");
        let mut agent_config = ResponsesConfig::new(required.clone());
        agent_config.optional_mut().set_store(Some(true));
        let agent = ResponsesAgent::builder(transport.clone())
            .with_raw_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent.clone());
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Responses Stream Stateful Step 1"));

        // Use on_raw_delta hook to extract response_id from raw stream events
        let captured_id = Arc::new(Mutex::new(None));
        let captured_id_clone = Arc::clone(&captured_id);

        let mut stream = runner.run_stream_with(&mut state, move || {
            let captured_id_inner = Arc::clone(&captured_id_clone);
            ChatStreamConfig::new().on_raw_delta(move |event: &ResponseStreamEvent| match event {
                ResponseStreamEvent::Created { response, .. }
                | ResponseStreamEvent::Completed { response, .. } => {
                    let mut guard = captured_id_inner.lock().unwrap();
                    if guard.is_none() {
                        *guard = Some(response.id.clone());
                    }
                }
                _ => {}
            })
        });

        let mut step1_text = String::new();
        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                step1_text.push_str(&text);
            }
        }

        assert_eq!(step1_text, "Responses Stream Stateful Step 1 Response");

        let previous_response_id = captured_id
            .lock()
            .unwrap()
            .take()
            .expect("Response ID should be captured via hook");
        assert_eq!(previous_response_id, "resp_stream_stateful_001");

        // Step 2: Discard first-round messages and set previous_response_id for next turn
        let mut updated_config = agent.config().clone();
        updated_config
            .optional_mut()
            .set_previous_response_id(Some(previous_response_id));
        let agent_step2 = ResponsesAgent::builder(transport)
            .with_raw_config(updated_config)
            .build()
            .unwrap();

        let runner_step2 = Runner::new(agent_step2);
        let mut state_step2 = ConversationState::new(None);
        state_step2.add_message(Message::user("Hello Responses Stream Stateful Step 2"));

        let mut stream_step2 = runner_step2.run_stream(&mut state_step2);
        let mut step2_text = String::new();

        while let Some(event_res) = stream_step2.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                step2_text.push_str(&text);
            }
        }

        assert_eq!(step2_text, "Responses Stream Stateful Step 2 Response");
    }

    #[tokio::test]
    async fn test_openai_tool_call_stream_e2e() {
        let guard = MockServerGuard::start(3004);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Tokyo?"));

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event
                && tc.name == "get_weather"
            {
                tool_called = true;
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
        let agent = MessagesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

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
        let agent = MessagesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in London?"));

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event
                && tc.name == "get_weather"
            {
                tool_called = true;
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
        let agent = GenerateContentAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

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
        let agent = GenerateContentAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Paris?"));

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event
                && tc.name == "get_weather"
            {
                tool_called = true;
            }
        }

        assert!(
            tool_called,
            "Tool 'get_weather' should be triggered for Gemini stream"
        );
    }

    #[tokio::test]
    async fn test_gemini_interactions_stream_e2e() {
        let guard = MockServerGuard::start(3023);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            InteractionsRequiredConfig::new("/v1beta/interactions").with_model("gemini-3.6-flash");
        let agent = InteractionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new(None);
        state.add_message(Message::user("Hello Gemini Interactions Stream!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert_eq!(received_text, "Hello from Gemini Interactions Stream Mock!");
    }

    #[tokio::test]
    async fn test_gemini_interactions_tool_call_stream_e2e() {
        let guard = MockServerGuard::start(3024);

        let transport = ReqwestTransport::new()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone());

        let agent_config =
            InteractionsRequiredConfig::new("/v1beta/interactions").with_model("gemini-3.6-flash");
        let agent = InteractionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent).with_tool(WeatherTool);
        let mut state = ConversationState::new(None);
        runner.sync_tools(&mut state);
        state.add_message(Message::user("What is the weather in Shanghai?"));

        let mut stream = runner.run_stream(&mut state);
        let mut tool_called = false;

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Event should be Ok");
            if let ChatStreamEvent::ToolCallFinished(tc) = event
                && tc.name == "get_weather"
            {
                tool_called = true;
            }
        }

        assert!(
            tool_called,
            "Tool 'get_weather' should be triggered for Gemini Interactions stream"
        );
    }
}
