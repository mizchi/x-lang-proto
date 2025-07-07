use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::commands::namespace::NamespaceManager;
use crate::commands::shell::NamespaceShell;

#[derive(Parser, Debug)]
#[command(name = "namespace", about = "Git-like namespace management")]
pub struct NamespaceCommand {
    #[command(subcommand)]
    command: Option<NamespaceSubcommand>,
}

#[derive(Subcommand, Debug)]
enum NamespaceSubcommand {
    /// Start interactive shell
    Shell,
    /// Show current namespace path
    Pwd,
    /// List entries in namespace
    Ls {
        /// Path to list (default: current)
        path: Option<String>,
    },
    /// Show function content
    Cat {
        /// Function name
        name: String,
    },
    /// Show specific version
    Show {
        /// Function name with hash (e.g., map#a1b2c3d4)
        spec: String,
    },
    /// Show function history
    Log {
        /// Function name
        name: String,
    },
    /// Export namespace to filesystem
    Export {
        /// Target directory
        path: PathBuf,
    },
    /// Import from filesystem
    Import {
        /// Source directory
        path: PathBuf,
    },
}

pub fn namespace_command(cmd: NamespaceCommand) -> anyhow::Result<()> {
    match cmd.command {
        None | Some(NamespaceSubcommand::Shell) => {
            // Start interactive shell
            let mut shell = NamespaceShell::new();
            shell.run().map_err(|e| anyhow::anyhow!(e))?;
        }
        Some(NamespaceSubcommand::Pwd) => {
            let mgr = NamespaceManager::new();
            println!("{}", mgr.pwd());
        }
        Some(NamespaceSubcommand::Ls { path }) => {
            let mut mgr = NamespaceManager::new();
            if let Some(p) = path {
                mgr.cd(&p).map_err(|e| anyhow::anyhow!(e))?;
            }
            let entries = mgr.ls();
            for entry in entries {
                println!("{}", entry);
            }
        }
        Some(NamespaceSubcommand::Cat { name }) => {
            let mgr = NamespaceManager::new();
            let content = mgr.cat(&name).map_err(|e| anyhow::anyhow!(e))?;
            println!("{}", content);
        }
        Some(NamespaceSubcommand::Show { spec }) => {
            if let Some((name, hash)) = spec.split_once('#') {
                let mgr = NamespaceManager::new();
                let content = mgr.show(name, hash).map_err(|e| anyhow::anyhow!(e))?;
                println!("{}", content);
            } else {
                return Err(anyhow::anyhow!("Invalid format. Use: name#hash"));
            }
        }
        Some(NamespaceSubcommand::Log { name }) => {
            let mgr = NamespaceManager::new();
            let history = mgr.log(&name).map_err(|e| anyhow::anyhow!(e))?;
            for entry in history {
                println!("{}", entry);
            }
        }
        Some(NamespaceSubcommand::Export { path }) => {
            let mgr = NamespaceManager::new();
            mgr.export(&path).map_err(|e| anyhow::anyhow!(e))?;
            println!("Exported to {}", path.display());
        }
        Some(NamespaceSubcommand::Import { path }) => {
            let mut mgr = NamespaceManager::new();
            mgr.import(&path).map_err(|e| anyhow::anyhow!(e))?;
            println!("Imported from {}", path.display());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        let cmd = NamespaceCommand::parse_from(&["namespace", "pwd"]);
        match cmd.command {
            Some(NamespaceSubcommand::Pwd) => {}
            _ => panic!("Expected Pwd command"),
        }
    }
}