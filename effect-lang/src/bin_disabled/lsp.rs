//! LSP server binary for x Language
//! 
//! This binary provides Language Server Protocol support for x Language,
//! enabling rich editor integration with real-time type checking,
//! effect inference, and intelligent code completion.

use effect_lang::{x LanguageuageServer, Result};
use lsp_server::{Connection, Message, Response};
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("effect_lang=info".parse().unwrap())
        )
        .init();

    info!("Starting x Language LSP server");

    // Check if we should run in stdio mode
    let args: Vec<String> = std::env::args().collect();
    let stdio_mode = args.iter().any(|arg| arg == "--stdio");

    if stdio_mode {
        run_stdio_server().await
    } else {
        run_tcp_server().await
    }
}

/// Run LSP server using stdio (standard mode for most editors)
async fn run_stdio_server() -> Result<()> {
    info!("Starting stdio LSP server");
    
    // Create the transport using stdin/stdout
    let (connection, io_threads) = Connection::stdio();
    
    // Run the main server loop
    let server_result = run_server(connection).await;
    
    // Wait for IO threads to complete
    io_threads.join()?;
    
    server_result
}

/// Run LSP server using TCP (useful for debugging)
async fn run_tcp_server() -> Result<()> {
    let port = std::env::var("EFFECT_LSP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    info!("Starting TCP LSP server on port {}", port);
    
    // This would require additional TCP transport implementation
    // For now, fall back to stdio
    warn!("TCP mode not yet implemented, falling back to stdio");
    run_stdio_server().await
}

/// Main server loop using lsp-server
async fn run_server(connection: Connection) -> Result<()> {
    // Server capabilities
    let server_capabilities = effect_lang::capabilities::server_capabilities();
    let server_capabilities = serde_json::to_value(server_capabilities)?;
    
    // Initialize the connection
    let initialize_params = connection.initialize(server_capabilities)?;
    let _initialize_params: lsp_types::InitializeParams = 
        serde_json::from_value(initialize_params)?;
    
    info!("LSP server initialized");
    
    // Create the language server instance
    let mut server = x LanguageuageServer::new();
    
    // Main message loop
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    info!("Received shutdown request");
                    break;
                }
                
                handle_request(&mut server, &connection, req).await?;
            }
            Message::Response(resp) => {
                warn!("Unexpected response: {:?}", resp);
            }
            Message::Notification(not) => {
                handle_notification(&mut server, not).await?;
            }
        }
    }
    
    info!("LSP server shutting down");
    Ok(())
}

/// Handle LSP requests
async fn handle_request(
    server: &mut x LanguageuageServer,
    connection: &Connection,
    req: lsp_server::Request,
) -> Result<()> {
    use lsp_types::request::*;
    
    let result = match req.method.as_str() {
        HoverRequest::METHOD => {
            let params: lsp_types::HoverParams = serde_json::from_value(req.params)?;
            let result = server.hover(params).await;
            serde_json::to_value(result)?
        }
        Completion::METHOD => {
            let params: lsp_types::CompletionParams = serde_json::from_value(req.params)?;
            let result = server.completion(params).await;
            serde_json::to_value(result)?
        }
        GotoDefinition::METHOD => {
            let params: lsp_types::GotoDefinitionParams = serde_json::from_value(req.params)?;
            let result = server.goto_definition(params).await;
            serde_json::to_value(result)?
        }
        References::METHOD => {
            let params: lsp_types::ReferenceParams = serde_json::from_value(req.params)?;
            let result = server.references(params).await;
            serde_json::to_value(result)?
        }
        DocumentSymbolRequest::METHOD => {
            let params: lsp_types::DocumentSymbolParams = serde_json::from_value(req.params)?;
            let result = server.document_symbol(params).await;
            serde_json::to_value(result)?
        }
        Formatting::METHOD => {
            let params: lsp_types::DocumentFormattingParams = serde_json::from_value(req.params)?;
            let result = server.formatting(params).await;
            serde_json::to_value(result)?
        }
        Rename::METHOD => {
            let params: lsp_types::RenameParams = serde_json::from_value(req.params)?;
            let result = server.rename(params).await;
            serde_json::to_value(result)?
        }
        CodeActionRequest::METHOD => {
            let params: lsp_types::CodeActionParams = serde_json::from_value(req.params)?;
            let result = server.code_action(params).await;
            serde_json::to_value(result)?
        }
        InlayHintRequest::METHOD => {
            let params: lsp_types::InlayHintParams = serde_json::from_value(req.params)?;
            let result = server.inlay_hint(params).await;
            serde_json::to_value(result)?
        }
        _ => {
            warn!("Unhandled request method: {}", req.method);
            serde_json::Value::Null
        }
    };
    
    let resp = Response::new_ok(req.id, result);
    connection.sender.send(Message::Response(resp))?;
    
    Ok(())
}

/// Handle LSP notifications
async fn handle_notification(
    server: &mut x LanguageuageServer,
    not: lsp_server::Notification,
) -> Result<()> {
    use lsp_types::notification::*;
    
    match not.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params: lsp_types::DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
            server.did_open(params).await;
        }
        DidChangeTextDocument::METHOD => {
            let params: lsp_types::DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
            server.did_change(params).await;
        }
        DidSaveTextDocument::METHOD => {
            let params: lsp_types::DidSaveTextDocumentParams = serde_json::from_value(not.params)?;
            server.did_save(params).await;
        }
        DidCloseTextDocument::METHOD => {
            let params: lsp_types::DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
            server.did_close(params).await;
        }
        Initialized::METHOD => {
            info!("Client confirmed initialization");
        }
        Exit::METHOD => {
            info!("Received exit notification");
        }
        _ => {
            warn!("Unhandled notification method: {}", not.method);
        }
    }
    
    Ok(())
}

/// Print usage information
fn print_usage() {
    println!("x Language LSP Server");
    println!();
    println!("USAGE:");
    println!("    effect-lsp [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("    --stdio    Use stdin/stdout for communication (default)");
    println!("    --help     Print this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("    EFFECT_LSP_PORT    TCP port for server (default: 8080)");
    println!("    RUST_LOG          Log level (default: info)");
    println!();
    println!("EXAMPLES:");
    println!("    # Run in stdio mode (typical editor usage)");
    println!("    effect-lsp --stdio");
    println!();
    println!("    # Enable debug logging");
    println!("    RUST_LOG=debug effect-lsp");
}