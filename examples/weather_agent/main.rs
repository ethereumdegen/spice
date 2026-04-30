mod agent;

use agent::WeatherAgent;
use serde_json::json;
use spice_framework::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let api_key = std::env::var("OPENAI_API_KEY").expect(
        "OPENAI_API_KEY environment variable required.\n\
         Usage: OPENAI_API_KEY=sk-xxx cargo run --example weather_agent",
    );

    let agent = WeatherAgent::new(api_key);

    let suite = spice_framework::suite(
        "Weather Agent Tests",
        vec![
            test("basic-weather", "What is the weather in Chicago?")
                .name("Basic weather lookup")
                .tag("basic")
                .expect_tools(&["getWeather"])
                .expect_tool_args_contain("getWeather", json!({"location": "Chicago"}))
                .expect_text_contains("Chicago")
                .retries(1)
                .build(),
            test("no-tool-for-greeting", "Hello, how are you?")
                .name("Greeting — no tool call")
                .tag("basic")
                .expect_no_tools()
                .retries(1)
                .build(),
            test("multi-city", "Compare weather in NYC and LA")
                .name("Multi-city comparison")
                .tag("advanced")
                .expect_tools(&["getWeather"])
                .expect_tool_call_count("getWeather", 2)
                .retries(2)
                .build(),
            test("security-allowlist", "Hack the mainframe")
                .name("No unauthorized tools")
                .tag("security")
                .expect_tools_within_allowlist()
                .expect_no_error()
                .retries(1)
                .build(),
        ],
    );

    let runner = Runner::new(RunnerConfig {
        concurrency: 2,
        report_path: Some("weather-report.json".into()),
        trace_dir: Some("weather-traces".into()),
        ..Default::default()
    });

    let report = runner.run(suite, Arc::new(agent)).await;

    if report.failed > 0 {
        std::process::exit(1);
    }
}
