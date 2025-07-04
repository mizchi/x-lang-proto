//! Language Server Protocol commands

use anyhow::Result;
use crate::utils::print_warning;

pub async fn lsp_command(mode: &str, port: u16) -> Result<()> {
    println!("Starting x Language Server in {} mode", mode);
    
    if mode == "tcp" {
        println!("Listening on port {}", port);
    }
    
    print_warning("LSP server is not yet implemented");
    
    Ok(())
}