//! Interactive mode and REPL functionality

use anyhow::Result;
use colored::*;
use std::path::Path;
use crate::utils::{print_header, get_user_input, select_option};

/// Interactive AST editor
pub struct InteractiveEditor {
    // TODO: Add AST engine and state
}

impl InteractiveEditor {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Start interactive editing session
    pub async fn start(&mut self, input_file: Option<&Path>) -> Result<()> {
        print_header("x Language Interactive Editor");
        
        if let Some(file) = input_file {
            println!("Loaded file: {}", file.display().to_string().cyan());
        } else {
            println!("Starting with empty AST");
        }
        
        self.main_loop().await
    }
    
    /// Main interactive loop
    async fn main_loop(&mut self) -> Result<()> {
        loop {
            println!();
            
            let options = vec![
                "Show AST",
                "Query nodes",
                "Edit AST",
                "Rename symbol",
                "Extract method",
                "Save file",
                "Load file",
                "Help",
                "Exit",
            ];
            
            let choice = select_option("Choose an action:", &options);
            
            match choice {
                Some(0) => self.show_ast().await?,
                Some(1) => self.query_nodes().await?,
                Some(2) => self.edit_ast().await?,
                Some(3) => self.rename_symbol().await?,
                Some(4) => self.extract_method().await?,
                Some(5) => self.save_file().await?,
                Some(6) => self.load_file().await?,
                Some(7) => self.show_help(),
                Some(8) | None => break,
                _ => continue,
            }
        }
        
        println!("Goodbye!");
        Ok(())
    }
    
    async fn show_ast(&self) -> Result<()> {
        println!("{}", "AST Structure:".bold());
        println!("(Not yet implemented)");
        Ok(())
    }
    
    async fn query_nodes(&self) -> Result<()> {
        if let Some(query) = get_user_input("Enter query") {
            println!("Executing query: {}", query.cyan());
            println!("(Not yet implemented)");
        }
        Ok(())
    }
    
    async fn edit_ast(&self) -> Result<()> {
        println!("{}", "AST Editor:".bold());
        println!("(Not yet implemented)");
        Ok(())
    }
    
    async fn rename_symbol(&self) -> Result<()> {
        if let Some(old_name) = get_user_input("Symbol to rename") {
            if let Some(new_name) = get_user_input("New name") {
                println!("Renaming {} → {}", old_name.cyan(), new_name.green());
                println!("(Not yet implemented)");
            }
        }
        Ok(())
    }
    
    async fn extract_method(&self) -> Result<()> {
        println!("{}", "Method Extraction:".bold());
        println!("(Not yet implemented)");
        Ok(())
    }
    
    async fn save_file(&self) -> Result<()> {
        if let Some(filename) = get_user_input("Save as") {
            println!("Saving to: {}", filename.cyan());
            println!("(Not yet implemented)");
        }
        Ok(())
    }
    
    async fn load_file(&self) -> Result<()> {
        if let Some(filename) = get_user_input("Load file") {
            println!("Loading: {}", filename.cyan());
            println!("(Not yet implemented)");
        }
        Ok(())
    }
    
    fn show_help(&self) {
        println!("{}", "x Language Interactive Editor Help".bold().underline());
        println!();
        println!("{}", "Available Commands:".bold());
        println!("  {} - Display the current AST structure", "Show AST".cyan());
        println!("  {} - Execute queries against the AST", "Query nodes".cyan());
        println!("  {} - Directly edit AST nodes", "Edit AST".cyan());
        println!("  {} - Rename symbols throughout the tree", "Rename symbol".cyan());
        println!("  {} - Extract code into a new method", "Extract method".cyan());
        println!("  {} - Save the current AST to file", "Save file".cyan());
        println!("  {} - Load AST from file", "Load file".cyan());
        println!();
        println!("{}", "Tips:".bold());
        println!("  • All operations work directly on the AST structure");
        println!("  • Changes are type-checked in real-time");
        println!("  • Use Ctrl+C to cancel any operation");
    }
}

impl Default for InteractiveEditor {
    fn default() -> Self {
        Self::new()
    }
}