use x_testing::{
    TestRunner, TestRunnerConfig,
    TestDiscovery,
    ConsoleReporter,
};
use x_editor::{
    namespace::{Namespace, NamespacePath},
    content_addressing::{ContentHash, ContentRepository},
};
use x_parser::Symbol;
use x_editor::namespace::Visibility;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("X Language Test Runner Demo");
    println!("===========================\n");
    
    // Create test configuration
    let config = TestRunnerConfig {
        cache_dir: PathBuf::from(".x-test-cache"),
        force_rerun: false,
        timeout_seconds: 60,
        num_threads: 1,
        verbose: true,
        filter: None,
    };
    
    // Create test runner
    let mut runner = TestRunner::new(config)?;
    
    // Create a test namespace
    let mut namespace = Namespace::new(NamespacePath::from_str("TestModule"));
    
    // Add some test functions
    add_test_function(&mut namespace, "test_pass", true);
    add_test_function(&mut namespace, "test_fail", false);
    add_test_function(&mut namespace, "test_another_pass", true);
    
    // Discover tests
    let content_repo = ContentRepository::new();
    let discovery = TestDiscovery::new(content_repo);
    let suite = discovery.discover_in_namespace(&namespace)?;
    
    println!("Discovered {} tests:\n", suite.tests.len());
    for test in &suite.tests {
        println!("  - {}", test.name);
    }
    println!();
    
    // Run tests
    let reporter = ConsoleReporter::new(true);
    let report = runner.run_suite(&suite, &reporter)?;
    
    // Show summary
    println!("\nTest Summary:");
    println!("  Total: {}", report.stats.total);
    println!("  Passed: {}", report.stats.passed);
    println!("  Failed: {}", report.stats.failed);
    println!("  Cached: {}", report.stats.cached);
    println!("  Cache hit rate: {:.1}%", report.cache_hit_rate());
    
    Ok(())
}

fn add_test_function(namespace: &mut Namespace, name: &str, should_pass: bool) {
    use x_editor::namespace::NameBinding;
    
    let test_hash = ContentHash::new(name.as_bytes());
    namespace.bindings.insert(
        Symbol::intern(name),
        NameBinding::Value {
            hash: test_hash,
            type_scheme: None,
            visibility: Visibility::Public,
        },
    );
}