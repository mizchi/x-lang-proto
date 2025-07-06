//! Test framework for x Language with content-addressed caching
//!
//! This module implements a Unison-style test runner where pure function tests
//! are cached by their content hash and only run once.

pub mod test_runner;
pub mod test_cache;
pub mod test_discovery;
pub mod test_report;

pub use test_runner::{TestRunner, TestRunnerConfig, TestResult};
pub use test_cache::{TestCache, CachedTestResult};
pub use test_discovery::{TestDiscovery, TestCase, TestSuite};
pub use test_report::{TestReport, TestReporter, ConsoleReporter};