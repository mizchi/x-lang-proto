//! Utility functions and helpers for the CLI

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    bar: ProgressBar,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        
        Self { bar }
    }
    
    /// Update the progress message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }
    
    /// Finish the progress indicator with a completion message
    pub fn finish(&self, message: &str) {
        self.bar.finish_with_message(format!("{} {}", "✓".green(), message));
    }
    
    /// Finish the progress indicator with an error message
    pub fn finish_error(&self, message: &str) {
        self.bar.finish_with_message(format!("{} {}", "✗".red(), message));
    }
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    
    if total_secs >= 60 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    } else if total_secs > 0 {
        format!("{}.{:03}s", total_secs, nanos / 1_000_000)
    } else if nanos >= 1_000_000 {
        format!("{:.1}ms", nanos as f64 / 1_000_000.0)
    } else if nanos >= 1_000 {
        format!("{:.1}μs", nanos as f64 / 1_000.0)
    } else {
        format!("{}ns", nanos)
    }
}

/// Print a styled header
pub fn print_header(title: &str) {
    println!();
    println!("{}", title.bold().underline());
    println!("{}", "─".repeat(title.len()));
}

/// Print a styled subheader
pub fn print_subheader(title: &str) {
    println!();
    println!("{}", title.bold());
}

/// Print an error message with consistent styling
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}

/// Print a warning message with consistent styling
pub fn print_warning(message: &str) {
    println!("{} {}", "Warning:".yellow().bold(), message);
}

/// Print a success message with consistent styling
pub fn print_success(message: &str) {
    println!("{} {}", "Success:".green().bold(), message);
}

/// Print an info message with consistent styling
pub fn print_info(message: &str) {
    println!("{} {}", "Info:".blue().bold(), message);
}

/// Confirm user action
pub fn confirm_action(message: &str) -> bool {
    use dialoguer::Confirm;
    
    Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Get user input
pub fn get_user_input(prompt: &str) -> Option<String> {
    use dialoguer::Input;
    
    Input::new()
        .with_prompt(prompt)
        .interact()
        .ok()
}

/// Select from multiple options
pub fn select_option(prompt: &str, options: &[&str]) -> Option<usize> {
    use dialoguer::Select;
    
    Select::new()
        .with_prompt(prompt)
        .items(options)
        .interact()
        .ok()
}

/// Create a table-like output
pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            column_widths: Vec::new(),
        }
    }
    
    pub fn headers(mut self, headers: Vec<&str>) -> Self {
        self.headers = headers.iter().map(|s| s.to_string()).collect();
        self.column_widths = headers.iter().map(|s| s.len()).collect();
        self
    }
    
    pub fn row(mut self, values: Vec<&str>) -> Self {
        let row: Vec<String> = values.iter().map(|s| s.to_string()).collect();
        
        // Update column widths
        for (i, value) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(value.len());
            } else {
                self.column_widths.push(value.len());
            }
        }
        
        self.rows.push(row);
        self
    }
    
    pub fn print(self) {
        if !self.headers.is_empty() {
            // Print headers
            for (i, header) in self.headers.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{:width$}", header.bold(), width = self.column_widths[i]);
            }
            println!();
            
            // Print separator
            for (i, &width) in self.column_widths.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{}", "─".repeat(width));
            }
            println!();
        }
        
        // Print rows
        for row in &self.rows {
            for (i, value) in row.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                let width = self.column_widths.get(i).copied().unwrap_or(0);
                print!("{:width$}", value, width = width);
            }
            println!();
        }
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_nanos(500)), "500ns");
        assert_eq!(format_duration(Duration::from_micros(1500)), "1.5μs");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.5ms");
        assert_eq!(format_duration(Duration::from_secs(1)), "1.000s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }
    
    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new()
            .headers(vec!["Name", "Age", "City"])
            .row(vec!["Alice", "25", "Tokyo"])
            .row(vec!["Bob", "30", "New York"]);
        
        // Just test that it doesn't panic
        table.print();
    }
}