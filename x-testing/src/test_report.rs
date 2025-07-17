//! Test reporting module
//!
//! This module provides various test reporters for displaying test results.

use std::collections::HashMap;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use x_editor::content_addressing::ContentHash;
use crate::test_runner::TestResult;
use crate::test_discovery::{TestCase, TestSuite};

/// Test report
#[derive(Debug)]
pub struct TestReport {
    /// Report name
    pub name: String,
    
    /// Test results
    pub results: HashMap<ContentHash, TestResult>,
    
    /// Total duration in milliseconds
    pub duration_ms: u64,
    
    /// Summary statistics
    pub stats: TestStats,
}

/// Test statistics
#[derive(Debug, Default)]
pub struct TestStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cached: usize,
}

impl TestReport {
    pub fn new(name: String) -> Self {
        Self {
            name,
            results: HashMap::new(),
            duration_ms: 0,
            stats: TestStats::default(),
        }
    }
    
    pub fn add_result(&mut self, test_hash: ContentHash, result: TestResult) {
        self.stats.total += 1;
        
        match &result {
            TestResult::Pass { .. } => self.stats.passed += 1,
            TestResult::Fail { .. } => self.stats.failed += 1,
            TestResult::Skipped { .. } => self.stats.skipped += 1,
            TestResult::Cached { original_result, .. } => {
                self.stats.cached += 1;
                match original_result.as_ref() {
                    TestResult::Pass { .. } => self.stats.passed += 1,
                    TestResult::Fail { .. } => self.stats.failed += 1,
                    TestResult::Skipped { .. } => self.stats.skipped += 1,
                    _ => {}
                }
            }
        }
        
        self.results.insert(test_hash, result);
    }
    
    pub fn is_success(&self) -> bool {
        self.stats.failed == 0
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        if self.stats.total == 0 {
            0.0
        } else {
            (self.stats.cached as f64) / (self.stats.total as f64) * 100.0
        }
    }
}

/// Test reporter trait
pub trait TestReporter {
    /// Called when a test suite starts
    fn on_suite_start(&self, suite: &TestSuite);
    
    /// Called when test count is determined
    fn on_test_count(&self, count: usize);
    
    /// Called when a test starts
    fn on_test_start(&self, test: &TestCase);
    
    /// Called when a test finishes
    fn on_test_finish(&self, test: &TestCase, result: &TestResult);
    
    /// Called when a test suite finishes
    fn on_suite_finish(&self, report: &TestReport);
}

/// Console test reporter
pub struct ConsoleReporter {
    verbose: bool,
    progress_bar: Option<ProgressBar>,
}

impl ConsoleReporter {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            progress_bar: None,
        }
    }
    
    fn print_test_result(&self, test: &TestCase, result: &TestResult) {
        let status = match result {
            TestResult::Pass { duration_ms, .. } => {
                format!("{} ({}ms)", "PASS".green(), duration_ms)
            }
            TestResult::Fail { duration_ms, error, .. } => {
                format!("{} ({}ms): {}", "FAIL".red(), duration_ms, error)
            }
            TestResult::Skipped { reason } => {
                format!("{}: {}", "SKIP".yellow(), reason)
            }
            TestResult::Cached { .. } => {
                format!("{}", "CACHED".cyan())
            }
        };
        
        println!("{} ... {}", test.full_path, status);
        
        if self.verbose {
            match result {
                TestResult::Pass { output: Some(output), .. } |
                TestResult::Fail { output: Some(output), .. } => {
                    println!("  Output: {}", output.dimmed());
                }
                _ => {}
            }
        }
    }
}

impl TestReporter for ConsoleReporter {
    fn on_suite_start(&self, suite: &TestSuite) {
        println!("\n{} {}\n", "Running".bold(), suite.name);
    }
    
    fn on_test_count(&self, count: usize) {
        if !self.verbose && count > 5 {
            let progress_bar = ProgressBar::new(count as u64);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-")
            );
            
            // Store progress bar for later use
            // Note: This is a simplified version. In real code, we'd use RefCell or similar
        }
    }
    
    fn on_test_start(&self, test: &TestCase) {
        if self.verbose {
            println!("Running {} ...", test.full_path);
        }
        
        if let Some(pb) = &self.progress_bar {
            pb.set_message(test.name.as_str().to_string());
        }
    }
    
    fn on_test_finish(&self, test: &TestCase, result: &TestResult) {
        if self.verbose || result.is_fail() {
            self.print_test_result(test, result);
        }
        
        if let Some(pb) = &self.progress_bar {
            pb.inc(1);
        }
    }
    
    fn on_suite_finish(&self, report: &TestReport) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
        
        println!("\n{}", "Test Summary".bold().underline());
        println!();
        
        let stats = &report.stats;
        
        if stats.passed > 0 {
            println!("  {} {}", stats.passed.to_string().green(), "passed");
        }
        if stats.failed > 0 {
            println!("  {} {}", stats.failed.to_string().red(), "failed");
        }
        if stats.skipped > 0 {
            println!("  {} {}", stats.skipped.to_string().yellow(), "skipped");
        }
        if stats.cached > 0 {
            println!("  {} {} ({}% cache hit rate)", 
                stats.cached.to_string().cyan(), 
                "cached",
                format!("{:.1}", report.cache_hit_rate())
            );
        }
        
        println!();
        println!("Total: {} tests in {:.2}s", 
            stats.total, 
            report.duration_ms as f64 / 1000.0
        );
        
        if report.is_success() {
            println!("\n{}", "All tests passed!".green().bold());
        } else {
            println!("\n{}", "Some tests failed.".red().bold());
            
            // List failed tests
            println!("\nFailed tests:");
            for (hash, result) in &report.results {
                if result.is_fail() {
                    if let TestResult::Fail { error, .. } = result {
                        println!("  - {}: {}", hash.0, error);
                    }
                }
            }
        }
    }
}

/// JSON test reporter
pub struct JsonReporter;

impl JsonReporter {
    pub fn new() -> Self {
        Self
    }
}

impl TestReporter for JsonReporter {
    fn on_suite_start(&self, _suite: &TestSuite) {
        // No action needed for JSON
    }
    
    fn on_test_count(&self, _count: usize) {
        // No action needed for JSON
    }
    
    fn on_test_start(&self, _test: &TestCase) {
        // No action needed for JSON
    }
    
    fn on_test_finish(&self, _test: &TestCase, _result: &TestResult) {
        // No action needed for JSON
    }
    
    fn on_suite_finish(&self, report: &TestReport) {
        let json_report = serde_json::json!({
            "name": report.name,
            "duration_ms": report.duration_ms,
            "stats": {
                "total": report.stats.total,
                "passed": report.stats.passed,
                "failed": report.stats.failed,
                "skipped": report.stats.skipped,
                "cached": report.stats.cached,
                "cache_hit_rate": report.cache_hit_rate(),
            },
            "success": report.is_success(),
            "results": report.results.iter().map(|(hash, result)| {
                (hash.0.clone(), match result {
                    TestResult::Pass { duration_ms, output } => serde_json::json!({
                        "status": "pass",
                        "duration_ms": duration_ms,
                        "output": output,
                    }),
                    TestResult::Fail { duration_ms, error, output } => serde_json::json!({
                        "status": "fail",
                        "duration_ms": duration_ms,
                        "error": error,
                        "output": output,
                    }),
                    TestResult::Skipped { reason } => serde_json::json!({
                        "status": "skipped",
                        "reason": reason,
                    }),
                    TestResult::Cached { .. } => serde_json::json!({
                        "status": "cached",
                    }),
                })
            }).collect::<HashMap<_, _>>(),
        });
        
        let json = serde_json::to_string_pretty(&json_report).unwrap();
        println!("{}", json);
    }
}

/// JUnit XML test reporter
pub struct JUnitReporter;

impl JUnitReporter {
    pub fn new() -> Self {
        Self
    }
}

impl TestReporter for JUnitReporter {
    fn on_suite_start(&self, _suite: &TestSuite) {
        // No action needed
    }
    
    fn on_test_count(&self, _count: usize) {
        // No action needed
    }
    
    fn on_test_start(&self, _test: &TestCase) {
        // No action needed
    }
    
    fn on_test_finish(&self, _test: &TestCase, _result: &TestResult) {
        // No action needed
    }
    
    fn on_suite_finish(&self, report: &TestReport) {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\" time=\"{}\">\n",
            report.name,
            report.stats.total,
            report.stats.failed,
            report.stats.skipped,
            report.duration_ms as f64 / 1000.0,
        ));
        
        for (hash, result) in &report.results {
            match result {
                TestResult::Pass { duration_ms, .. } => {
                    xml.push_str(&format!(
                        "  <testcase name=\"{}\" time=\"{}\" />\n",
                        hash.0,
                        *duration_ms as f64 / 1000.0,
                    ));
                }
                TestResult::Fail { duration_ms, error, .. } => {
                    xml.push_str(&format!(
                        "  <testcase name=\"{}\" time=\"{}\">\n",
                        hash.0,
                        *duration_ms as f64 / 1000.0,
                    ));
                    xml.push_str(&format!("    <failure message=\"{}\" />\n", error));
                    xml.push_str("  </testcase>\n");
                }
                TestResult::Skipped { reason } => {
                    xml.push_str(&format!(
                        "  <testcase name=\"{}\">\n",
                        hash.0,
                    ));
                    xml.push_str(&format!("    <skipped message=\"{}\" />\n", reason));
                    xml.push_str("  </testcase>\n");
                }
                TestResult::Cached { .. } => {
                    // Treat cached as pass for JUnit
                    xml.push_str(&format!(
                        "  <testcase name=\"{}\" time=\"0\" />\n",
                        hash.0,
                    ));
                }
            }
        }
        
        xml.push_str("</testsuite>\n");
        
        print!("{}", xml);
    }
}

impl JsonReporter {
    pub fn new_stdout() -> Self {
        Self::new()
    }
}

impl JUnitReporter {
    pub fn new_stdout() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_stats() {
        let mut report = TestReport::new("Test Suite".to_string());
        
        let hash1 = ContentHash::new(b"test1");
        let hash2 = ContentHash::new(b"test2");
        
        report.add_result(hash1, TestResult::Pass { 
            duration_ms: 100, 
            output: None 
        });
        
        report.add_result(hash2, TestResult::Fail {
            duration_ms: 50,
            error: "Assertion failed".to_string(),
            output: None,
        });
        
        assert_eq!(report.stats.total, 2);
        assert_eq!(report.stats.passed, 1);
        assert_eq!(report.stats.failed, 1);
        assert!(!report.is_success());
    }
}