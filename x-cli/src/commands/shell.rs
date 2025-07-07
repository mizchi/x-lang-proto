use std::io::{self, Write};
use std::path::PathBuf;
use crate::commands::namespace::NamespaceManager;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};

/// Interactive shell for namespace management
pub struct NamespaceShell {
    manager: NamespaceManager,
    editor_command: String,
}

impl NamespaceShell {
    pub fn new() -> Self {
        Self {
            manager: NamespaceManager::new(),
            editor_command: std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string()),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut rl = DefaultEditor::new().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e.to_string())
        })?;
        
        println!("x-lang namespace shell v0.1.0");
        println!("Type 'help' for available commands\n");

        loop {
            let prompt = format!("{}> ", self.manager.pwd());
            let readline = rl.readline(&prompt);
            
            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }
                    
                    match self.execute_command(&parts) {
                        Ok(should_exit) => {
                            if should_exit {
                                break;
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, parts: &[&str]) -> Result<bool, String> {
        match parts[0] {
            "help" | "?" => {
                self.print_help();
                Ok(false)
            }
            "exit" | "quit" => Ok(true),
            "pwd" => {
                println!("{}", self.manager.pwd());
                Ok(false)
            }
            "cd" => {
                if parts.len() < 2 {
                    self.manager.cd("/")?;
                } else {
                    self.manager.cd(parts[1])?;
                }
                Ok(false)
            }
            "ls" => {
                let entries = self.manager.ls();
                if entries.is_empty() {
                    println!("(empty)");
                } else {
                    for entry in entries {
                        println!("{}", entry);
                    }
                }
                Ok(false)
            }
            "cat" => {
                if parts.len() < 2 {
                    return Err("Usage: cat <function>".to_string());
                }
                let content = self.manager.cat(parts[1])?;
                println!("{}", content);
                Ok(false)
            }
            "show" => {
                if parts.len() < 2 {
                    return Err("Usage: show <function>#<hash>".to_string());
                }
                let arg = parts[1];
                if let Some((name, hash)) = arg.split_once('#') {
                    let content = self.manager.show(name, hash)?;
                    println!("{}", content);
                } else {
                    return Err("Usage: show <function>#<hash>".to_string());
                }
                Ok(false)
            }
            "edit" => {
                if parts.len() < 2 {
                    return Err("Usage: edit <function>".to_string());
                }
                self.edit_function(parts[1])?;
                Ok(false)
            }
            "log" => {
                if parts.len() < 2 {
                    return Err("Usage: log <function>".to_string());
                }
                let history = self.manager.log(parts[1])?;
                for entry in history {
                    println!("{}", entry);
                }
                Ok(false)
            }
            "mkdir" => {
                if parts.len() < 2 {
                    return Err("Usage: mkdir <namespace>".to_string());
                }
                self.create_namespace(parts[1])?;
                Ok(false)
            }
            "export" => {
                if parts.len() < 2 {
                    return Err("Usage: export <directory>".to_string());
                }
                let path = PathBuf::from(parts[1]);
                self.manager.export(&path)?;
                println!("Exported to {}", path.display());
                Ok(false)
            }
            "import" => {
                if parts.len() < 2 {
                    return Err("Usage: import <directory>".to_string());
                }
                let path = PathBuf::from(parts[1]);
                self.manager.import(&path)?;
                println!("Imported from {}", path.display());
                Ok(false)
            }
            "deps" => {
                if parts.len() < 2 {
                    return Err("Usage: deps <function>".to_string());
                }
                // TODO: Implement dependency analysis
                println!("Dependency analysis not yet implemented");
                Ok(false)
            }
            cmd => Err(format!("Unknown command: {}. Type 'help' for available commands.", cmd)),
        }
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  help, ?           Show this help message");
        println!("  exit, quit        Exit the shell");
        println!("  pwd               Show current namespace path");
        println!("  cd <path>         Change namespace");
        println!("  ls                List entries in current namespace");
        println!("  cat <function>    Show function content");
        println!("  show <func>#<hash> Show specific version");
        println!("  edit <function>   Edit function");
        println!("  log <function>    Show function history");
        println!("  mkdir <namespace> Create new namespace");
        println!("  export <dir>      Export namespace to filesystem");
        println!("  import <dir>      Import from filesystem");
        println!("  deps <function>   Show dependencies");
        println!();
        println!("Navigation:");
        println!("  cd /              Go to root");
        println!("  cd ..             Go to parent");
        println!("  cd Core/List      Go to specific path");
    }

    fn edit_function(&mut self, name: &str) -> Result<(), String> {
        use std::process::Command;
        use tempfile::NamedTempFile;

        // Get current content if exists
        let current_content = self.manager.cat(name).unwrap_or_default();

        // Create temporary file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        
        temp_file.write_all(current_content.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        let temp_path = temp_file.path().to_path_buf();
        
        // Open editor
        let status = Command::new(&self.editor_command)
            .arg(&temp_path)
            .status()
            .map_err(|e| format!("Failed to launch editor: {}", e))?;

        if !status.success() {
            return Err("Editor exited with error".to_string());
        }

        // Read edited content
        let new_content = std::fs::read_to_string(&temp_path)
            .map_err(|e| format!("Failed to read edited file: {}", e))?;

        // Save if changed
        if let Some(hash) = self.manager.edit(name, new_content)? {
            println!("Committed {}#{}", name, &hash.0[..8]);
        } else {
            println!("No changes");
        }

        Ok(())
    }

    fn create_namespace(&mut self, name: &str) -> Result<(), String> {
        // Save current path
        let current_path = self.manager.pwd();
        
        // Create namespace by creating a dummy entry
        self.manager.cd("/")?; // Go to root to ensure we can navigate back
        
        // Navigate to the parent of the new namespace
        let parts: Vec<&str> = name.split('/').collect();
        let (parent_parts, namespace_name) = if parts.len() > 1 {
            let mut parent = parts.clone();
            let name = parent.pop().unwrap();
            (parent, name)
        } else {
            (vec![], parts[0])
        };
        
        // Navigate to parent
        for part in parent_parts {
            self.manager.cd(part)?;
        }
        
        // Create a dummy function to establish the namespace
        self.manager.edit(&format!("{}/.keep", namespace_name), "# Namespace placeholder".to_string())?;
        
        // Navigate back
        self.manager.cd(&current_path)?;
        
        println!("Created namespace: {}", name);
        Ok(())
    }
}

// Extension trait for Hash to get short version
trait HashExt {
    fn short(&self) -> String;
}

impl HashExt for super::namespace::Hash {
    fn short(&self) -> String {
        self.0[..8].to_string()
    }
}