pub mod agent;
pub mod assertion;
pub mod error;
pub mod mock;
pub mod multi_turn;
pub mod rbac;
pub mod report;
pub mod runner;
pub mod test_case;
pub mod toolkit;
pub mod trace;

pub use agent::{AgentConfig, AgentOutput, AgentUnderTest, ToolCall, Turn};
pub use assertion::Assertion;
pub use error::SpiceError;
pub use mock::{MockAgent, MockMultiTurnResponse, MockResponse, MockTurn};
pub use rbac::RbacMatrix;
pub use report::{SuiteReport, TestReport};
pub use runner::{Runner, RunnerConfig};
pub use test_case::{TestCase, TestCaseBuilder, TestSuite};
pub use toolkit::{ParamDef, PromptTemplate, ToolDef, Toolkit};

/// Convenience function to start building a test case.
pub fn test(id: impl Into<String>, user_message: impl Into<String>) -> TestCaseBuilder {
    TestCaseBuilder::new(id, user_message)
}

/// Convenience function to create a test suite.
pub fn suite(name: impl Into<String>, tests: Vec<TestCase>) -> TestSuite {
    TestSuite {
        name: name.into(),
        tests,
        ..Default::default()
    }
}
