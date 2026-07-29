"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createMockServer = createMockServer;
const aimock_1 = require("@copilotkit/aimock");
function createMockServer(port) {
    const mock = new aimock_1.LLMock({ port: port || 0 });
    // 1. OpenAI Tool Call match (e.g. "What is the weather in Tokyo?")
    mock.on({ userMessage: /weather/i }, {
        toolCalls: [
            {
                name: 'get_weather',
                arguments: JSON.stringify({ location: 'Tokyo', unit: 'celsius' }),
                id: 'call_weather_123',
            },
        ],
    });
    // 2. Gemini Stream & Non-stream matches
    mock.onMessage(/Hello Gemini Stream/i, {
        content: 'Hello from Gemini Stream Mock!',
    });
    mock.onMessage(/Hello Gemini Non-Stream/i, {
        content: 'Hello from Gemini Mock!',
    });
    mock.onMessage(/Hello Gemini!/i, {
        content: 'Hello from Gemini Stream Mock!',
    });
    // 3. Claude matches
    mock.onMessage(/Hello Claude/i, {
        content: 'Hello from Claude Mock!',
    });
    // 4. OpenAI matches
    mock.onMessage(/Hello OpenAI/i, {
        content: 'Hello! How can I help you today?',
    });
    // 5. Fallback match for any other prompt
    mock.onMessage(/.*/, {
        content: 'Hello! How can I help you today?',
    });
    return mock;
}
