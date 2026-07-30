/// Stream E2E tests for `oxide-llm`.
///
/// `oxide-llm` 流式 (Stream) 端到端 E2E 测试模块。
#[cfg(test)]
mod tests {
    use crate::{MockServerGuard, WeatherTool};
    use futures::StreamExt;
    use oxide_llm::{
        Runner, TransportBuilder,
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
        },
        executor::TokioToolRegistry,
        proto::openai::v1::response::ResponseStreamEvent,
        transport::reqwest::ReqwestTransport,
    };

    #[tokio::test]
    async fn test_openai_chat_completions_stream_e2e() {
        let guard = MockServerGuard::start(3001);

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config = ResponsesRequiredConfig::new("gpt-4o", "/v1/responses");
        let agent = ResponsesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let required = ResponsesRequiredConfig::new("gpt-4o", "/v1/responses");
        let mut agent_config = ResponsesConfig::new(required.clone());
        agent_config.optional_mut().set_store(Some(true));
        let agent = ResponsesAgent::builder(transport.clone())
            .with_raw_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent.clone());
        let mut state = ConversationState::new();
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
        let mut state_step2 = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let registry = TokioToolRegistry::new().with_tool(WeatherTool);
        let runner = Runner::new(agent).with_registry(registry);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let registry = TokioToolRegistry::new().with_tool(WeatherTool);
        let runner = Runner::new(agent).with_registry(registry);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            GenerateContentRequiredConfig::new("gemini-1.5-pro", "/v1beta/models/gemini-1.5-pro");
        let agent = GenerateContentAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            GenerateContentRequiredConfig::new("gemini-1.5-pro", "/v1beta/models/gemini-1.5-pro");
        let agent = GenerateContentAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let registry = TokioToolRegistry::new().with_tool(WeatherTool);
        let runner = Runner::new(agent).with_registry(registry);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            InteractionsRequiredConfig::new("/v1beta/interactions").with_model("gemini-3.6-flash");
        let agent = InteractionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
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

        let transport = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();

        let agent_config =
            InteractionsRequiredConfig::new("/v1beta/interactions").with_model("gemini-3.6-flash");
        let agent = InteractionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let registry = TokioToolRegistry::new().with_tool(WeatherTool);
        let runner = Runner::new(agent).with_registry(registry);
        let mut state = ConversationState::new();
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

    // ────────────────────────────────────────────────────────────────────────
    // AnyTransport stream E2E tests
    // ────────────────────────────────────────────────────────────────────────

    /// Verify that `AnyTransport` wrapping `ReqwestTransport` + middleware
    /// correctly proxies the byte stream through the erased interface.
    ///
    /// 验证 `AnyTransport` 包装后，字节流能完整地通过 erased 接口传递，
    /// 流式输出与具体 Transport 的结果一致。
    #[tokio::test]
    async fn test_any_transport_openai_stream_e2e() {
        use oxide_llm::core::transport::AnyTransport;

        let guard = MockServerGuard::start(3040);

        let concrete = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();
        let transport = AnyTransport::new(concrete);

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
        state.add_message(Message::user("Hello OpenAI!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("AnyTransport stream event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert!(
            !received_text.is_empty(),
            "AnyTransport should forward streamed text from mock OpenAI server"
        );
    }

    /// Verify that a cloned `AnyTransport` streams the same data as the
    /// original — the `Arc` sharing must not break stream independence.
    ///
    /// 验证克隆的 `AnyTransport` 与原始实例在流式模式下行为一致，
    /// `Arc` 共享不能破坏流的独立性。
    #[tokio::test]
    async fn test_any_transport_clone_stream_e2e() {
        use oxide_llm::core::transport::AnyTransport;

        let guard = MockServerGuard::start(3041);

        let concrete = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();
        let transport = AnyTransport::new(concrete);
        let transport_clone = transport.clone();

        let agent_config = ChatCompletionsRequiredConfig::new("gpt-4o", "/v1/chat/completions");
        let agent = ChatCompletionsAgent::builder(transport_clone)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
        state.add_message(Message::user("Hello OpenAI!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("Cloned AnyTransport stream event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert!(
            !received_text.is_empty(),
            "Cloned AnyTransport should also forward streamed text correctly"
        );
    }

    /// Verify that `AnyTransport` works with Claude agent in stream mode.
    ///
    /// 验证 `AnyTransport` 对 Claude 代理在流式模式下正常工作。
    #[tokio::test]
    async fn test_any_transport_claude_stream_e2e() {
        use oxide_llm::core::transport::AnyTransport;

        let guard = MockServerGuard::start(3042);

        let concrete = ReqwestTransport::builder()
            .with_authorization("mock-api-key")
            .with_base_url(guard.base_url.clone())
            .build()
            .unwrap();
        let transport = AnyTransport::new(concrete);

        let agent_config =
            MessagesRequiredConfig::new("claude-3-5-sonnet-20240620", 1024, "/v1/messages");
        let agent = MessagesAgent::builder(transport)
            .with_required_config(agent_config)
            .build()
            .unwrap();

        let runner = Runner::new(agent);
        let mut state = ConversationState::new();
        state.add_message(Message::user("Hello Claude!"));

        let mut stream = runner.run_stream(&mut state);
        let mut received_text = String::new();

        while let Some(event_res) = stream.next().await {
            let event = event_res.expect("AnyTransport Claude stream event should be Ok");
            if let ChatStreamEvent::Text { text } = event {
                received_text.push_str(&text);
            }
        }

        assert_eq!(received_text, "Hello from Claude Mock!");
    }
}
