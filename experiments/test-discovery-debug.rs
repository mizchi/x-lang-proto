use x_testing::{TestDiscovery, TestRunner, TestRunnerConfig, ConsoleReporter};
use x_editor::{
    namespace::{Namespace, NamespacePath},
    content_addressing::ContentRepository,
};
use x_parser::{parse_source, FileId, SyntaxStyle};
use x_checker::TypeChecker;
use std::fs;

#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::compilation_unit_to_namespace;

fn main() -> anyhow::Result<()> {
    let content = fs::read_to_string("simple_test.x")?;
    
    // Parse the file
    let file_id = FileId(0);
    let compilation_unit = parse_source(&content, file_id, SyntaxStyle::RustLike)?;
    
    println!("Parsed module: {}", compilation_unit.module.name.to_string());
    println!("Items in module: {}", compilation_unit.module.items.len());
    
    // Show all value definitions
    for item in &compilation_unit.module.items {
        if let x_parser::Item::ValueDef(value_def) = item {
            println!("  Function: {}", value_def.name);
            if let Some(type_ann) = &value_def.type_annotation {
                println!("    Type annotation: {:?}", type_ann);
            }
        }
    }
    
    // Type check
    let mut type_checker = TypeChecker::new();
    let check_result = type_checker.check_compilation_unit(&compilation_unit);
    
    println!("\nType check errors: {}", check_result.errors.len());
    println!("Inferred types: {}", check_result.inferred_types.len());
    
    // Convert to namespace
    let namespace_path = NamespacePath::from_str("simple_test");
    let namespace = compilation_unit_to_namespace(&compilation_unit, namespace_path, &check_result)?;
    
    println!("\nNamespace bindings: {}", namespace.bindings.len());
    for (name, binding) in &namespace.bindings {
        println!("  Binding: {}", name);
        if let x_editor::namespace::NameBinding::Value { .. } = binding {
            println!("    -> Is a value binding");
        }
    }
    
    // Test discovery
    let content_repo = ContentRepository::new();
    let discovery = TestDiscovery::new(content_repo.clone());
    let suite = discovery.discover_in_namespace(&namespace)?;
    
    println!("\nDiscovered tests: {}", suite.tests.len());
    for test in &suite.tests {
        println!("  Test: {} ({})", test.name, test.full_path);
    }
    
    // Run tests
    if !suite.tests.is_empty() {
        let config = TestRunnerConfig::default();
        let mut runner = TestRunner::new(config)?;
        let reporter = ConsoleReporter::new(true);
        let report = runner.run_suite(&suite, &reporter)?;
        
        println!("\nTest results:");
        println!("  Passed: {}", report.stats.passed);
        println!("  Failed: {}", report.stats.failed);
    }
    
    Ok(())
}