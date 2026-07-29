import { LLMock } from '@copilotkit/aimock';

export function createMockServer(port?: number): LLMock {
  const mock = new LLMock({ port: port || 0 });

  // 1. OpenAI Tool Call match
  mock.on(
    { userMessage: /weather in Tokyo/i },
    {
      toolCalls: [
        {
          name: 'get_weather',
          arguments: JSON.stringify({ location: 'Tokyo', unit: 'celsius' }),
          id: 'call_weather_123',
        },
      ],
    }
  );

  mock.on(
    { userMessage: /weather in Berlin/i },
    {
      toolCalls: [
        {
          name: 'get_weather',
          arguments: JSON.stringify({ location: 'Berlin', unit: 'celsius' }),
          id: 'call_weather_berlin',
        },
      ],
    }
  );

  mock.on(
    { userMessage: /weather in London/i },
    {
      toolCalls: [
        {
          name: 'get_weather',
          arguments: JSON.stringify({ location: 'London', unit: 'celsius' }),
          id: 'call_weather_london',
        },
      ],
    }
  );

  mock.on(
    { userMessage: /weather in Paris/i },
    {
      toolCalls: [
        {
          name: 'get_weather',
          arguments: JSON.stringify({ location: 'Paris', unit: 'celsius' }),
          id: 'call_weather_paris',
        },
      ],
    }
  );

  // 2. Multi-turn tool execution loop match (Beijing)
  // Step 1: user asks for weather without tool result -> returns tool call
  mock.on(
    { userMessage: /Multi-turn weather in Beijing/i, hasToolResult: false },
    {
      toolCalls: [
        {
          name: 'get_weather',
          arguments: JSON.stringify({ location: 'Beijing', unit: 'celsius' }),
          id: 'call_beijing_999',
        },
      ],
    }
  );

  // Step 2: request has tool result -> returns final text answer
  mock.on(
    { hasToolResult: true },
    {
      content: 'The weather in Beijing is 20°C with sunny skies.',
    }
  );

  // 3. System prompt match
  mock.on(
    { systemMessage: /you are a helpful assistant with system prompt/i },
    {
      content: 'System prompt acknowledged!',
    }
  );

  // 4. OpenAI Responses Agent matches
  mock.onMessage(/Hello Responses Non-Stream/i, {
    content: 'Hello from OpenAI Responses Non-Stream Mock!',
  });

  mock.onMessage(/Hello Responses Stream/i, {
    content: 'Hello from OpenAI Responses Stream Mock!',
  });

  // 5. Gemini Stream & Non-stream matches
  mock.onMessage(/Hello Gemini Stream/i, {
    content: 'Hello from Gemini Stream Mock!',
  });

  mock.onMessage(/Hello Gemini Non-Stream/i, {
    content: 'Hello from Gemini Mock!',
  });

  mock.onMessage(/Hello Gemini!/i, {
    content: 'Hello from Gemini Stream Mock!',
  });

  // 6. Claude matches
  mock.onMessage(/Hello Claude Non-Stream/i, {
    content: 'Hello from Claude Mock!',
  });

  mock.onMessage(/Hello Claude/i, {
    content: 'Hello from Claude Mock!',
  });

  // 7. OpenAI matches
  mock.onMessage(/Hello OpenAI Non-Stream/i, {
    content: 'Hello! How can I help you today?',
  });

  mock.onMessage(/Hello OpenAI/i, {
    content: 'Hello! How can I help you today?',
  });

  // 8. Error handling matches
  mock.on(
    { userMessage: /Trigger 401 Unauthorized/i },
    {
      status: 401,
      error: {
        message: 'Invalid API Key',
        type: 'invalid_request_error',
        code: 'invalid_api_key',
      },
    }
  );

  mock.on(
    { userMessage: /Trigger 500 Internal Error/i },
    {
      status: 500,
      error: {
        message: 'Internal server error occurred',
        type: 'api_error',
        code: 'internal_error',
      },
    }
  );

  // 9. Fallback match for any other prompt
  mock.onMessage(/.*/, {
    content: 'Hello! How can I help you today?',
  });

  return mock;
}
