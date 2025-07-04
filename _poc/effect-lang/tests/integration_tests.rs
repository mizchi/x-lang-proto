//! Integration tests for x Language CLI and end-to-end workflows

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_cli_path() -> PathBuf {
    let mut path = std::env::current_dir().expect("Should get current dir");
    path.push("target");
    path.push("release");
    path.push("effect-cli");
    path
}

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Should write test file");
    path
}

#[test]
fn test_cli_compile_command() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    let source_content = r#"
module Test
let x = 42
let add = fun a b -> a + b
"#;
    
    let source_file = create_test_file(&temp_dir, "test.eff", source_content);
    let binary_file = temp_dir.path().join("test.eff.bin");
    
    let output = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&source_file)
        .arg("-o").arg(&binary_file)
        .arg("--timing")
        .output()
        .expect("Should run CLI command");
    
    if !output.status.success() {
        panic!("CLI compile failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Check that binary file was created
    assert!(binary_file.exists(), "Binary file should be created");
    
    // Check that binary file has content
    let binary_data = fs::read(&binary_file).expect("Should read binary file");
    assert!(!binary_data.is_empty(), "Binary file should not be empty");
    
    // Check output contains expected information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled"), "Should show compilation success");
    assert!(stdout.contains("Binary size"), "Should show binary size");
    assert!(stdout.contains("Content hash"), "Should show content hash");
}

#[test]
fn test_cli_analyze_command() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    let source_content = r#"
module Analysis
let fibonacci = fun n ->
  if n <= 1 then n
  else fibonacci (n - 1) + fibonacci (n - 2)
"#;
    
    let source_file = create_test_file(&temp_dir, "analysis.eff", source_content);
    let binary_file = temp_dir.path().join("analysis.eff.bin");
    
    // First compile
    let compile_output = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&source_file)
        .arg("-o").arg(&binary_file)
        .output()
        .expect("Should compile");
    
    assert!(compile_output.status.success(), "Compilation should succeed");
    
    // Then analyze
    let analyze_output = Command::new(get_cli_path())
        .arg("analyze")
        .arg(&binary_file)
        .arg("--hash")
        .arg("--size")
        .output()
        .expect("Should run analyze command");
    
    if !analyze_output.status.success() {
        panic!("CLI analyze failed: {}", String::from_utf8_lossy(&analyze_output.stderr));
    }
    
    let stdout = String::from_utf8_lossy(&analyze_output.stdout);
    assert!(stdout.contains("Analysis of"), "Should show analysis header");
    assert!(stdout.contains("Binary size"), "Should show binary size");
    assert!(stdout.contains("Content hash"), "Should show content hash");
    assert!(stdout.contains("AST Statistics"), "Should show AST statistics");
}

#[test]
fn test_cli_diff_command() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    let content1 = r#"
module DiffTest
let x = 42
let add = fun a b -> a + b
"#;
    
    let content2 = r#"
module DiffTest
let x = 43
let add = fun a b -> a + b
"#;
    
    let file1 = create_test_file(&temp_dir, "test1.eff", content1);
    let file2 = create_test_file(&temp_dir, "test2.eff", content2);
    
    let binary1 = temp_dir.path().join("test1.eff.bin");
    let binary2 = temp_dir.path().join("test2.eff.bin");
    
    // Compile both files
    for (source, binary) in [(file1, binary1.clone()), (file2, binary2.clone())] {
        let output = Command::new(get_cli_path())
            .arg("compile")
            .arg("-i").arg(source)
            .arg("-o").arg(binary)
            .output()
            .expect("Should compile");
        
        assert!(output.status.success(), "Compilation should succeed");
    }
    
    // Run diff
    let diff_output = Command::new(get_cli_path())
        .arg("diff")
        .arg(&binary1)
        .arg(&binary2)
        .arg("--verbose")
        .output()
        .expect("Should run diff command");
    
    if !diff_output.status.success() {
        panic!("CLI diff failed: {}", String::from_utf8_lossy(&diff_output.stderr));
    }
    
    let stdout = String::from_utf8_lossy(&diff_output.stdout);
    assert!(stdout.contains("Diff between"), "Should show diff header");
    // Should detect the change from 42 to 43
    assert!(!stdout.contains("= ") || stdout.contains("~") || stdout.contains("+") || stdout.contains("-"), 
            "Should show some kind of difference");
}

#[test]
fn test_identical_files_diff() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    let content = r#"
module Identical
let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;
    
    let file1 = create_test_file(&temp_dir, "identical1.eff", content);
    let file2 = create_test_file(&temp_dir, "identical2.eff", content);
    
    let binary1 = temp_dir.path().join("identical1.eff.bin");
    let binary2 = temp_dir.path().join("identical2.eff.bin");
    
    // Compile both
    for (source, binary) in [(file1, binary1.clone()), (file2, binary2.clone())] {
        let output = Command::new(get_cli_path())
            .arg("compile")
            .arg("-i").arg(source)
            .arg("-o").arg(binary)
            .output()
            .expect("Should compile");
        
        assert!(output.status.success(), "Compilation should succeed");
    }
    
    // Diff should show no differences
    let diff_output = Command::new(get_cli_path())
        .arg("diff")
        .arg(&binary1)
        .arg(&binary2)
        .output()
        .expect("Should run diff command");
    
    assert!(diff_output.status.success(), "Diff should succeed");
    
    let stdout = String::from_utf8_lossy(&diff_output.stdout);
    // Should indicate equality in some way
    assert!(stdout.contains("= ") || stdout.is_empty() || stdout.contains("identical"), 
            "Should indicate files are equal or very similar");
}

#[test]
fn test_compression_efficiency() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    // Create a reasonably large source file
    let mut content = String::from("module LargeModule\n");
    for i in 0..100 {
        content.push_str(&format!("let function{} = fun x{} -> x{} + {}\n", i, i, i, i));
    }
    
    let source_file = create_test_file(&temp_dir, "large.eff", &content);
    let binary_file = temp_dir.path().join("large.eff.bin");
    
    let output = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&source_file)
        .arg("-o").arg(&binary_file)
        .output()
        .expect("Should compile");
    
    assert!(output.status.success(), "Large file compilation should succeed");
    
    let source_size = fs::metadata(&source_file).expect("Should get source metadata").len();
    let binary_size = fs::metadata(&binary_file).expect("Should get binary metadata").len();
    
    let compression_ratio = binary_size as f64 / source_size as f64;
    
    println!("Source size: {} bytes", source_size);
    println!("Binary size: {} bytes", binary_size);
    println!("Compression ratio: {:.2}x", compression_ratio);
    
    // Binary should be reasonably sized
    assert!(binary_size > 0, "Binary should have content");
    assert!(compression_ratio < 5.0, "Binary shouldn't be more than 5x source size");
    
    // Check that output reports this information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compression ratio"), "Should report compression ratio");
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    // Test with malformed source
    let bad_content = r#"
module Test
let x = 
let y = 42
"#;
    
    let bad_file = create_test_file(&temp_dir, "bad.eff", bad_content);
    
    let output = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&bad_file)
        .output()
        .expect("Should run command");
    
    // Should fail gracefully
    assert!(!output.status.success(), "Should fail on malformed input");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("error"), "Should show error message");
}

#[test]
fn test_nonexistent_file() {
    let output = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg("nonexistent.eff")
        .output()
        .expect("Should run command");
    
    // Should fail gracefully
    assert!(!output.status.success(), "Should fail on nonexistent file");
}

#[test]
fn test_help_command() {
    let output = Command::new(get_cli_path())
        .arg("--help")
        .output()
        .expect("Should show help");
    
    assert!(output.status.success(), "Help should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compile"), "Should show compile command");
    assert!(stdout.contains("diff"), "Should show diff command");
    assert!(stdout.contains("analyze"), "Should show analyze command");
}

#[test]
fn test_workflow_simulation() {
    // Simulate a typical development workflow
    let temp_dir = TempDir::new().expect("Should create temp dir");
    
    // 1. Create initial version
    let v1_content = r#"
module Calculator
let add = fun x y -> x + y
let multiply = fun x y -> x * y
"#;
    
    let v1_file = create_test_file(&temp_dir, "calc_v1.eff", v1_content);
    let v1_binary = temp_dir.path().join("calc_v1.eff.bin");
    
    // 2. Compile v1
    let compile_v1 = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&v1_file)
        .arg("-o").arg(&v1_binary)
        .output()
        .expect("Should compile v1");
    
    assert!(compile_v1.status.success(), "V1 compilation should succeed");
    
    // 3. Create modified version
    let v2_content = r#"
module Calculator
let add = fun x y -> x + y
let multiply = fun x y -> x * y
let subtract = fun x y -> x - y
"#;
    
    let v2_file = create_test_file(&temp_dir, "calc_v2.eff", v2_content);
    let v2_binary = temp_dir.path().join("calc_v2.eff.bin");
    
    // 4. Compile v2
    let compile_v2 = Command::new(get_cli_path())
        .arg("compile")
        .arg("-i").arg(&v2_file)
        .arg("-o").arg(&v2_binary)
        .output()
        .expect("Should compile v2");
    
    assert!(compile_v2.status.success(), "V2 compilation should succeed");
    
    // 5. Diff the versions
    let diff_output = Command::new(get_cli_path())
        .arg("diff")
        .arg(&v1_binary)
        .arg(&v2_binary)
        .arg("--verbose")
        .output()
        .expect("Should diff versions");
    
    assert!(diff_output.status.success(), "Diff should succeed");
    
    // 6. Analyze both versions
    for (name, binary) in [("v1", &v1_binary), ("v2", &v2_binary)] {
        let analyze = Command::new(get_cli_path())
            .arg("analyze")
            .arg(binary)
            .arg("--hash")
            .output()
            .expect("Should analyze");
        
        assert!(analyze.status.success(), "Analysis of {} should succeed", name);
        
        let stdout = String::from_utf8_lossy(&analyze.stdout);
        assert!(stdout.contains("Calculator"), "Should show module name");
    }
    
    println!("Workflow simulation completed successfully");
}