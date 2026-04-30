use crate::agent::{AgentConfig, AgentUnderTest};
use crate::assertion::AssertionResult;
use crate::report::{SuiteReport, TestReport};
use crate::test_case::{TestCase, TestSuite};
use crate::trace::Trace;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Configuration for the test runner.
pub struct RunnerConfig {
    /// Max concurrent tests.
    pub concurrency: usize,
    /// Default timeout per test.
    pub default_timeout: Duration,
    /// Filter: only run tests whose id or name contains this substring.
    pub filter: Option<String>,
    /// Filter: only run tests with any of these tags.
    pub tag_filter: Option<Vec<String>>,
    /// Directory to write trace files.
    pub trace_dir: Option<PathBuf>,
    /// Path to write JSON report.
    pub report_path: Option<PathBuf>,
    /// Print console output.
    pub console_output: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            default_timeout: Duration::from_secs(60),
            filter: None,
            tag_filter: None,
            trace_dir: None,
            report_path: None,
            console_output: true,
        }
    }
}

/// The test runner.
pub struct Runner {
    pub config: RunnerConfig,
}

impl Runner {
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Run a test suite against an agent, returning the suite report.
    pub async fn run(
        &self,
        suite: TestSuite,
        agent: Arc<dyn AgentUnderTest>,
    ) -> SuiteReport {
        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));

        let tests: Vec<TestCase> = suite
            .tests
            .into_iter()
            .filter(|t| self.matches_filter(t))
            .collect();

        let total = tests.len();
        let mut handles = Vec::with_capacity(total);

        for test_case in tests {
            let sem = semaphore.clone();
            let agent = agent.clone();
            let default_timeout = suite
                .default_timeout
                .unwrap_or(self.config.default_timeout);
            let default_retries = suite.default_retries;
            let default_config = suite.default_config.clone();
            let trace_dir = self.config.trace_dir.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                run_single_test(
                    test_case,
                    &*agent,
                    default_timeout,
                    default_retries,
                    &default_config,
                    trace_dir.as_ref(),
                )
                .await
            });
            handles.push(handle);
        }

        let mut reports = Vec::with_capacity(total);
        for handle in handles {
            match handle.await {
                Ok(report) => reports.push(report),
                Err(e) => {
                    reports.push(TestReport {
                        test_id: "unknown".into(),
                        test_name: None,
                        tags: vec![],
                        passed: false,
                        attempts: 0,
                        assertion_results: vec![],
                        duration: Duration::ZERO,
                        error: Some(format!("Task panicked: {}", e)),
                    });
                }
            }
        }

        let passed = reports.iter().filter(|r| r.passed).count();
        let failed = reports.len() - passed;

        let suite_report = SuiteReport {
            suite_name: suite.name,
            tests: reports,
            total,
            passed,
            failed,
            duration: start.elapsed(),
            timestamp: chrono::Utc::now(),
        };

        if self.config.console_output {
            suite_report.print_console();
        }

        if let Some(path) = &self.config.report_path {
            if let Err(e) = suite_report.save_to_file(path) {
                eprintln!("Failed to save report: {}", e);
            }
        }

        suite_report
    }

    fn matches_filter(&self, test: &TestCase) -> bool {
        if let Some(filter) = &self.config.filter {
            let id_match = test.id.contains(filter.as_str());
            let name_match = test
                .name
                .as_ref()
                .map(|n| n.contains(filter.as_str()))
                .unwrap_or(false);
            if !id_match && !name_match {
                return false;
            }
        }
        if let Some(tag_filter) = &self.config.tag_filter {
            if !test.tags.iter().any(|t| tag_filter.contains(t)) {
                return false;
            }
        }
        true
    }
}

async fn run_single_test(
    test: TestCase,
    agent: &dyn AgentUnderTest,
    default_timeout: Duration,
    default_retries: usize,
    default_config: &AgentConfig,
    trace_dir: Option<&PathBuf>,
) -> TestReport {
    let start = Instant::now();
    let timeout = test.timeout.unwrap_or(default_timeout);
    let max_retries = test.retries.max(default_retries);
    let config = if test.config.data.is_null() {
        default_config
    } else {
        &test.config
    };
    let available_tools = agent.available_tools(config);

    // Consensus mode
    if let (Some(runs), Some(required)) = (test.consensus_runs, test.consensus_required) {
        return run_consensus(
            &test,
            agent,
            config,
            &available_tools,
            timeout,
            runs,
            required,
            trace_dir,
            start,
        )
        .await;
    }

    // Standard retry mode
    let mut last_results = vec![];
    let mut last_error = None;
    let mut attempts = 0;

    for attempt in 0..=max_retries {
        attempts = attempt + 1;

        let run_result = tokio::time::timeout(timeout, agent.run(&test.user_message, config)).await;

        match run_result {
            Ok(Ok(output)) => {
                // Save trace
                if let Some(dir) = trace_dir {
                    let trace = Trace::new(
                        test.id.clone(),
                        test.user_message.clone(),
                        output.clone(),
                    );
                    let path = dir.join(format!("{}_attempt{}.json", test.id, attempt));
                    let _ = trace.save_to_file(&path);
                }

                let results: Vec<AssertionResult> = test
                    .assertions
                    .iter()
                    .map(|a| a.evaluate(&output, &available_tools))
                    .collect();

                let all_passed = results.iter().all(|r| r.passed);
                if all_passed {
                    return TestReport {
                        test_id: test.id,
                        test_name: test.name,
                        tags: test.tags,
                        passed: true,
                        attempts,
                        assertion_results: results,
                        duration: start.elapsed(),
                        error: None,
                    };
                }
                last_results = results;
                last_error = None;
            }
            Ok(Err(e)) => {
                last_error = Some(e.to_string());
                last_results = vec![];
            }
            Err(_) => {
                last_error = Some(format!("Timeout after {:?}", timeout));
                last_results = vec![];
            }
        }
    }

    TestReport {
        test_id: test.id,
        test_name: test.name,
        tags: test.tags,
        passed: false,
        attempts,
        assertion_results: last_results,
        duration: start.elapsed(),
        error: last_error,
    }
}

async fn run_consensus(
    test: &TestCase,
    agent: &dyn AgentUnderTest,
    config: &AgentConfig,
    available_tools: &[String],
    timeout: Duration,
    runs: usize,
    required: usize,
    trace_dir: Option<&PathBuf>,
    start: Instant,
) -> TestReport {
    let mut pass_count = 0;
    let mut last_results = vec![];

    for i in 0..runs {
        let run_result = tokio::time::timeout(timeout, agent.run(&test.user_message, config)).await;

        match run_result {
            Ok(Ok(output)) => {
                if let Some(dir) = trace_dir {
                    let trace = Trace::new(
                        test.id.clone(),
                        test.user_message.clone(),
                        output.clone(),
                    );
                    let path = dir.join(format!("{}_consensus{}.json", test.id, i));
                    let _ = trace.save_to_file(&path);
                }

                let results: Vec<AssertionResult> = test
                    .assertions
                    .iter()
                    .map(|a| a.evaluate(&output, available_tools))
                    .collect();

                let all_passed = results.iter().all(|r| r.passed);
                if all_passed {
                    pass_count += 1;
                }
                last_results = results;
            }
            Ok(Err(_)) | Err(_) => {
                // Count as failure
            }
        }

        if pass_count >= required {
            break;
        }
    }

    TestReport {
        test_id: test.id.clone(),
        test_name: test.name.clone(),
        tags: test.tags.clone(),
        passed: pass_count >= required,
        attempts: runs,
        assertion_results: last_results,
        duration: start.elapsed(),
        error: if pass_count < required {
            Some(format!(
                "Consensus: {}/{} passed, needed {}",
                pass_count, runs, required
            ))
        } else {
            None
        },
    }
}
