use std::fs;
use std::path::Path;

fn main() -> std::io::Result<()> {
    println!("Searching for .x files in current directory:");
    
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "x") {
            println!("  Found: {}", path.display());
            
            // Check if it's a valid x file
            if let Ok(content) = fs::read_to_string(&path) {
                if content.starts_with("module") {
                    println!("    ✓ Has module declaration");
                    
                    // Count test functions
                    let test_count = content.lines()
                        .filter(|line| line.contains("test_") || line.contains("test"))
                        .count();
                    println!("    → Potential test functions: {}", test_count);
                }
            }
        }
    }
    
    Ok(())
}