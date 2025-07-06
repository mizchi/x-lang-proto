//! Test command implementation
//!
//! This module implements the test runner command for x Language,
//! integrating with the content-addressed caching test framework.

use anyhow::{Result, Context};
use colored::*;
use std::path::Path;
use std::io;
use x_testing::{
    TestRunner, TestRunnerConfig, 
    TestDiscovery, TestSuite,
    ConsoleReporter, TestReporter,
    test_report::{JsonReporter, JUnitReporter},
};
use x_editor::{
    namespace::{Namespace, NamespacePath},
    namespace_storage::NamespaceStorage,
    content_addressing::ContentRepository,
};
use x_parser::{parse_source, FileId, SyntaxStyle};
use x_checker::TypeChecker;
use std::fs;
use crate::commands::test_helpers::compilation_unit_to_namespace;

/// Run tests command
pub async fn test_command(
    path: &Path,
    filter: Option<&str>,
    force: bool,
    threads: Option<usize>,
    verbose: bool,
    reporter: &str,
    timeout: u64,
) -> Result<()> {
    println!("{} {}", "Running tests in".cyan(), path.display());
    
    // Create test runner configuration
    let config = TestRunnerConfig {
        cache_dir: path.join(".x-test-cache"),
        force_rerun: force,
        timeout_seconds: timeout,
        num_threads: threads.unwrap_or_else(num_cpus::get),
        verbose,
        filter: filter.map(String::from),
    };
    
    // Initialize components
    let content_repo = ContentRepository::new();
    let namespace_storage = NamespaceStorage::new(path.join(".x-namespaces"), content_repo.clone())?;
    let mut type_checker = TypeChecker::new();
    
    // Discover tests
    let discovery = TestDiscovery::new(content_repo.clone());
    let suite = discover_tests(path, &discovery, &namespace_storage, &mut type_checker).await?;
    
    if suite.tests.is_empty() {
        println!("{}", "No tests found!".yellow());
        return Ok(());
    }
    
    println!("Found {} tests", suite.tests.len());
    
    // Create test runner
    let mut runner = TestRunner::new(config)?;
    
    // Create reporter
    let reporter: Box<dyn TestReporter> = match reporter {
        "json" => Box::new(JsonReporter::new_stdout()),
        "junit" => Box::new(JUnitReporter::new_stdout()),
        _ => Box::new(ConsoleReporter::new(verbose)),
    };
    
    // Run tests
    let report = runner.run_suite(&suite, reporter.as_ref())?;
    
    // Exit with appropriate code
    if report.is_success() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

async fn discover_tests(
    path: &Path,
    discovery: &TestDiscovery,
    namespace_storage: &NamespaceStorage,
    type_checker: &mut TypeChecker,
) -> Result<TestSuite> {
    if path.is_file() {
        // Single file test
        discover_file_tests(path, discovery, namespace_storage, type_checker).await
    } else {
        // Directory test discovery
        discover_directory_tests(path, discovery, namespace_storage, type_checker).await
    }
}

async fn discover_file_tests(
    path: &Path,
    discovery: &TestDiscovery,
    _namespace_storage: &NamespaceStorage,
    type_checker: &mut TypeChecker,
) -> Result<TestSuite> {
    let content = fs::read_to_string(path)
        .context("Failed to read test file")?;
    
    // Parse the file
    let file_id = FileId(0);
    let compilation_unit = parse_source(&content, file_id, SyntaxStyle::RustLike)
        .context("Failed to parse test file")?;
    
    // Type check
    let check_result = type_checker.check_compilation_unit(&compilation_unit);
    
    // Create a temporary namespace
    let namespace_path = NamespacePath::from_str(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("test")
    );
    
    // Convert the compilation unit to a namespace with test functions
    let namespace = compilation_unit_to_namespace(&compilation_unit, namespace_path, &check_result)?;
    
    // Discover tests in the namespace
    discovery.discover_in_namespace(&namespace)
}

async fn discover_directory_tests(
    path: &Path,
    discovery: &TestDiscovery,
    namespace_storage: &NamespaceStorage,
    type_checker: &mut TypeChecker,
) -> Result<TestSuite> {
    let mut namespaces = Vec::new();
    
    // Walk directory for .x files
    for entry in walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "x") {
            if let Ok(namespace) = load_namespace_from_file(path, namespace_storage, type_checker).await {
                namespaces.push(namespace);
            }
        }
    }
    
    // Check for stored namespaces
    let namespace_paths = namespace_storage.list_namespaces();
    for _namespace_path in namespace_paths {
        // Note: load_namespace requires mutable access to namespace_storage
        // For now, skip loading from storage
        // if let Ok(namespace) = namespace_storage.load_namespace(&namespace_path) {
        //     namespaces.push(namespace);
        // }
    }
    
    // Discover tests in all namespaces
    discovery.discover_in_namespaces(&namespaces)
}

async fn load_namespace_from_file(
    path: &Path,
    _namespace_storage: &NamespaceStorage,
    type_checker: &mut TypeChecker,
) -> Result<Namespace> {
    let content = fs::read_to_string(path)
        .context("Failed to read file")?;
    
    let file_id = FileId(0);
    let compilation_unit = parse_source(&content, file_id, SyntaxStyle::RustLike)
        .context("Failed to parse file")?;
    
    let check_result = type_checker.check_compilation_unit(&compilation_unit);
    
    let namespace_path = NamespacePath::from_str(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    );
    
    // Convert the compilation unit to a namespace
    compilation_unit_to_namespace(&compilation_unit, namespace_path, &check_result)
}

// Missing dependencies
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

mod walkdir {
    use std::path::{Path, PathBuf};
    use std::fs;
    
    pub struct WalkDir {
        root: PathBuf,
        follow_links: bool,
    }
    
    impl WalkDir {
        pub fn new<P: AsRef<Path>>(path: P) -> Self {
            Self {
                root: path.as_ref().to_path_buf(),
                follow_links: false,
            }
        }
        
        pub fn follow_links(mut self, follow: bool) -> Self {
            self.follow_links = follow;
            self
        }
        
        pub fn into_iter(self) -> impl Iterator<Item = Result<DirEntry, std::io::Error>> {
            WalkDirIter::new(self.root, self.follow_links)
        }
    }
    
    pub struct DirEntry {
        path: PathBuf,
    }
    
    impl DirEntry {
        pub fn path(&self) -> &Path {
            &self.path
        }
    }
    
    struct WalkDirIter {
        stack: Vec<PathBuf>,
        follow_links: bool,
    }
    
    impl WalkDirIter {
        fn new(root: PathBuf, follow_links: bool) -> Self {
            Self {
                stack: vec![root],
                follow_links,
            }
        }
    }
    
    impl Iterator for WalkDirIter {
        type Item = Result<DirEntry, std::io::Error>;
        
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(path) = self.stack.pop() {
                if path.is_dir() {
                    match fs::read_dir(&path) {
                        Ok(entries) => {
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    self.stack.push(entry.path());
                                }
                            }
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                
                return Some(Ok(DirEntry { path }));
            }
            
            None
        }
    }
}