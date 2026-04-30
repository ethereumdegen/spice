use crate::agent::AgentOutput;
use crate::error::SpiceError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A trace record for a single test execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub test_id: String,
    pub user_message: String,
    pub output: AgentOutput,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Trace {
    pub fn new(test_id: String, user_message: String, output: AgentOutput) -> Self {
        Self {
            test_id,
            user_message,
            output,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Save trace to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), SpiceError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
