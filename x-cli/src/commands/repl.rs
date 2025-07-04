//! REPL commands

use anyhow::Result;
use std::path::Path;
use crate::utils::print_warning;

pub async fn repl_command(preload: Option<&Path>, syntax: &str) -> Result<()> {
    println!("Starting x Language REPL with {} syntax", syntax);
    
    if let Some(path) = preload {
        println!("Preloading: {}", path.display());
    }
    
    print_warning("REPL is not yet implemented");
    
    Ok(())
}