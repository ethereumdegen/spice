use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::SpiceError;

/// Configuration passed to the agent under test. Wraps arbitrary JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    pub data: serde_json::Value,
}

impl AgentConfig {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }

    pub fn empty() -> Self {
        Self {
            data: serde_json::Value::Null,
        }
    }
}

/// A single tool call made by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// A single turn in the agent's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub index: usize,
    pub output_text: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<serde_json::Value>,
    pub stop_reason: Option<String>,
    pub duration: Duration,
}

/// The complete output of an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub final_text: String,
    pub turns: Vec<Turn>,
    pub tools_called: Vec<String>,
    pub duration: Duration,
    pub error: Option<String>,
}

impl AgentOutput {
    /// Collect all tool calls across all turns.
    pub fn all_tool_calls(&self) -> Vec<&ToolCall> {
        self.turns
            .iter()
            .flat_map(|t| t.tool_calls.iter())
            .collect()
    }

    /// Get tool calls filtered by name.
    pub fn tool_calls_by_name(&self, name: &str) -> Vec<&ToolCall> {
        self.all_tool_calls()
            .into_iter()
            .filter(|tc| tc.name == name)
            .collect()
    }
}

/// Trait that the agent under test must implement.
#[async_trait]
pub trait AgentUnderTest: Send + Sync {
    /// Run the agent with a user message and config, return full output with trace.
    async fn run(&self, user_message: &str, config: &AgentConfig)
        -> Result<AgentOutput, SpiceError>;

    /// Return tool names available for this config (for allowlist assertions).
    fn available_tools(&self, config: &AgentConfig) -> Vec<String>;

    /// Human-readable agent name (for reports).
    fn name(&self) -> &str {
        "agent"
    }
}
