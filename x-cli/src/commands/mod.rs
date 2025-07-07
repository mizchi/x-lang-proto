//! Command implementations for x CLI


pub mod new;
pub mod convert;
pub mod show;
pub mod query;
pub mod edit;
// pub mod rename;
pub mod extract;
pub mod hash;
pub mod check;
pub mod compile;
pub mod repl;
pub mod lsp;
pub mod stats;
pub mod test;
pub mod test_helpers;
pub mod doc;
pub mod version;
pub mod resolve;
pub mod imports;
pub mod outdated;
pub mod namespace;
pub mod namespace_cli;
pub mod shell;

// Re-export command functions
pub use new::new_command;
pub use convert::convert_command;
pub use show::show_command;
pub use query::query_command;
pub use edit::edit_command;
// pub use rename::rename_command;
pub use extract::ExtractArgs;
pub use check::check_command;
pub use compile::compile_command;
pub use repl::repl_command;
pub use lsp::lsp_command;
pub use stats::stats_command;
pub use test::test_command;
pub use doc::DocCommand;
pub use namespace_cli::{NamespaceCommand, namespace_command};