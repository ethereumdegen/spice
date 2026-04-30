use crate::assertion::AssertionResult;
use crate::error::SpiceError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Result of a single test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub test_id: String,
    pub test_name: Option<String>,
    pub tags: Vec<String>,
    pub passed: bool,
    pub attempts: usize,
    pub assertion_results: Vec<AssertionResult>,
    pub duration: Duration,
    pub error: Option<String>,
}

/// Result of an entire test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteReport {
    pub suite_name: String,
    pub tests: Vec<TestReport>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SuiteReport {
    /// Print colored console output.
    pub fn print_console(&self) {
        println!();
        println!(
            "  \x1b[1m{}\x1b[0m  ({} tests)",
            self.suite_name, self.total
        );
        println!("  {}", "─".repeat(50));

        for test in &self.tests {
            let display_name = test
                .test_name
                .as_deref()
                .unwrap_or(&test.test_id);

            if test.passed {
                println!("  \x1b[32m✓ PASS\x1b[0m  {}", display_name);
            } else {
                println!("  \x1b[31m✗ FAIL\x1b[0m  {}", display_name);
                for ar in &test.assertion_results {
                    if !ar.passed {
                        let prefix = if ar.is_security {
                            "\x1b[33m🔒\x1b[0m"
                        } else {
                            " "
                        };
                        println!(
                            "          {} {}",
                            prefix,
                            ar.message.as_deref().unwrap_or(&ar.description)
                        );
                    }
                }
                if let Some(err) = &test.error {
                    println!("          error: {}", err);
                }
            }
        }

        // --- RBAC summary ---
        let rbac_tests: Vec<_> = self
            .tests
            .iter()
            .filter(|t| t.tags.iter().any(|tag| tag == "rbac"))
            .collect();

        if !rbac_tests.is_empty() {
            println!("  {}", "─".repeat(50));
            println!("  \x1b[1mRBAC Summary\x1b[0m");

            // Group by role (extracted from test_id pattern rbac-*-ROLE or test name)
            let mut role_results: std::collections::BTreeMap<String, (usize, usize)> =
                std::collections::BTreeMap::new();
            for t in &rbac_tests {
                // Try to extract role from test name "ROLE — ..." or test_id
                let role = t
                    .test_name
                    .as_deref()
                    .and_then(|n| n.split(" — ").next())
                    .unwrap_or(&t.test_id)
                    .to_string();
                let entry = role_results.entry(role).or_insert((0, 0));
                entry.0 += 1;
                if t.passed {
                    entry.1 += 1;
                }
            }

            for (role, (total, passed)) in &role_results {
                let color = if passed == total {
                    "\x1b[32m"
                } else {
                    "\x1b[31m"
                };
                println!(
                    "    {}{}: {}/{} passed\x1b[0m",
                    color, role, passed, total
                );
            }

            let rbac_passed = rbac_tests.iter().filter(|t| t.passed).count();
            let rbac_total = rbac_tests.len();
            let rbac_color = if rbac_passed == rbac_total {
                "\x1b[32m"
            } else {
                "\x1b[31m"
            };
            println!(
                "  {}RBAC Total: {}/{} passed\x1b[0m",
                rbac_color, rbac_passed, rbac_total
            );
        }

        println!("  {}", "─".repeat(50));

        let security_tests: Vec<_> = self
            .tests
            .iter()
            .filter(|t| {
                t.assertion_results.iter().any(|a| a.is_security)
            })
            .collect();

        if !security_tests.is_empty() {
            let sec_passed = security_tests.iter().filter(|t| t.passed).count();
            let sec_total = security_tests.len();
            let color = if sec_passed == sec_total {
                "\x1b[32m"
            } else {
                "\x1b[31m"
            };
            println!(
                "  {}Security: {}/{} passed\x1b[0m",
                color, sec_passed, sec_total
            );
        }

        let color = if self.failed == 0 {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        };
        println!(
            "  {}Total: {}/{} passed\x1b[0m  ({:.1}s)",
            color,
            self.passed,
            self.total,
            self.duration.as_secs_f64()
        );
        println!();
    }

    /// Save report to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), SpiceError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
