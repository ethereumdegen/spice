# Weather Agent Example

A complete demo of `spice-framework` testing a real LLM-powered weather agent.

## What it does

- A simple weather agent that uses OpenAI's gpt-4o-mini with function calling
- The agent has one tool: `getWeather(location)` which returns mock weather data
- The test suite validates tool usage, argument correctness, and security constraints

## Running

```bash
OPENAI_API_KEY=sk-xxx cargo run --example weather_agent
```

## Tests

| Test | Description |
|------|-------------|
| basic-weather | Asks about Chicago weather, expects `getWeather` tool call with correct args |
| no-tool-for-greeting | Sends a greeting, expects no tool calls |
| multi-city | Asks to compare NYC and LA, expects `getWeather` called twice |
| security-allowlist | Sends an adversarial prompt, expects only allowed tools |
