// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI print utilities — mirrors claude-code-typescript-src `cli/print.ts`.
// Provides styled output for CLI commands.

/// ANSI color codes for terminal output.
pub mod colors {
    use std::fmt;

    pub struct Red(pub String);
    pub struct Green(pub String);
    pub struct Yellow(pub String);
    pub struct Bold(pub String);

    impl fmt::Display for Red {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[31m{}\x1b[0m", self.0)
        }
    }

    impl fmt::Display for Green {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[32m{}\x1b[0m", self.0)
        }
    }

    impl fmt::Display for Yellow {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[33m{}\x1b[0m", self.0)
        }
    }

    impl fmt::Display for Bold {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "\x1b[1m{}\x1b[0m", self.0)
        }
    }

    /// Wrap text in red color.
    pub fn red(text: &str) -> String {
        format!("\x1b[31m{}\x1b[0m", text)
    }

    /// Wrap text in green color.
    pub fn green(text: &str) -> String {
        format!("\x1b[32m{}\x1b[0m", text)
    }

    /// Wrap text in yellow color.
    pub fn yellow(text: &str) -> String {
        format!("\x1b[33m{}\x1b[0m", text)
    }

    /// Wrap text in bold.
    pub fn bold(text: &str) -> String {
        format!("\x1b[1m{}\x1b[0m", text)
    }

    /// Check if terminal supports colors.
    pub fn supports_color() -> bool {
        std::env::var("NO_COLOR").is_err()
    }
}

/// Output figures for visual indicators.
pub mod figures {
    /// Check mark.
    pub const TICK: &str = "✓";
    /// Cross mark.
    pub const CROSS: &str = "✗";
    /// Warning indicator.
    pub const WARNING: &str = "⚠";
    /// Pointer for lists.
    pub const POINTER: &str = "▸";
    /// Bullet for lists.
    pub const BULLET: &str = "•";
}

/// Write styled output to stdout.
pub fn write_stdout(msg: &str) {
    print!("{}", msg);
}

/// Write styled output to stdout with newline.
pub fn writeln_stdout(msg: &str) {
    println!("{}", msg);
}

/// Write styled output to stderr.
pub fn write_stderr(msg: &str) {
    eprint!("{}", msg);
}

/// Write styled output to stderr with newline.
pub fn writeln_stderr(msg: &str) {
    eprintln!("{}", msg);
}

/// Print success message with tick.
pub fn success(msg: &str) {
    println!("{} {}", figures::TICK, msg);
}

/// Print error message with cross.
pub fn error(msg: &str) {
    eprintln!("{} {}", figures::CROSS, msg);
}

/// Print warning message.
pub fn warning(msg: &str) {
    eprintln!("{} {}", figures::WARNING, msg);
}

/// Print info message.
pub fn info(msg: &str) {
    println!("  {}", msg);
}

/// Print a list item with pointer.
pub fn list_item(msg: &str) {
    println!("{} {}", figures::POINTER, msg);
}

/// Print a bullet item.
pub fn bullet(msg: &str) {
    println!("{} {}", figures::BULLET, msg);
}

/// Print a key-value pair with formatting.
pub fn kv(key: &str, value: &str) {
    println!("  {}: {}", colors::yellow(key), value);
}

/// Print a section header.
pub fn section(header: &str) {
    println!("\n{}", colors::bold(header));
}

/// Print a sub-section.
pub fn subsection(header: &str) {
    println!("\n{}", header);
}

/// Format a table row.
pub fn table_row(cols: &[&str], widths: &[usize]) -> String {
    cols.iter()
        .zip(widths.iter())
        .map(|(col, &width)| format!("{:<width$}", col, width = width))
        .collect::<Vec<_>>()
        .join("  ")
}

/// Print a table with columns.
pub fn print_table(headers: &[&str], rows: Vec<Vec<&str>>, widths: Option<Vec<usize>>) {
    let calculated_widths = widths.unwrap_or_else(|| {
        let mut w = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        for row in &rows {
            for (i, col) in row.iter().enumerate() {
                if i < w.len() {
                    w[i] = w[i].max(col.len());
                }
            }
        }
        w
    });

    // Print header
    println!("{}", table_row(headers, &calculated_widths));

    // Print separator
    let sep: String = calculated_widths
        .iter()
        .map(|&w| "─".repeat(w))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", sep);

    // Print rows
    for row in rows {
        println!("{}", table_row(&row, &calculated_widths));
    }
}

/// Progress indicator for long-running operations.
pub struct Progress {
    current: usize,
    total: usize,
    message: String,
}

impl Progress {
    pub fn new(total: usize, message: &str) -> Self {
        Self {
            current: 0,
            total,
            message: message.to_string(),
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
        self.print();
    }

    pub fn print(&self) {
        let pct = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as usize
        } else {
            0
        };
        print!(
            "\r{}: {}/{} ({}%) ",
            self.message, self.current, self.total, pct
        );
    }

    pub fn finish(self) {
        println!();
    }
}

/// Spinner for indeterminate progress.
pub struct Spinner {
    message: String,
    chars: &'static [char],
    current: usize,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            chars: &['|', '/', '-', '\\', '|', '/', '-', '\\'],
            current: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current = (self.current + 1) % self.chars.len();
        print!("\r{} {}...", self.chars[self.current], self.message);
    }

    pub fn finish(self, final_message: &str) {
        println!("\r{} {}", figures::TICK, final_message);
    }
}

/// Format bytes as human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human-readable string.
pub fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

/// JSON output mode for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text output.
    Text,
    /// JSON output.
    Json,
    /// JSON lines output.
    JsonLines,
}

impl OutputFormat {
    pub fn from_env() -> Self {
        if std::env::var("JSON").is_ok() {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        }
    }
}

/// Print data in specified format.
pub fn print_in_format<T: serde::Serialize + std::fmt::Debug>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("{:#?}", data);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
        }
        OutputFormat::JsonLines => {
            println!("{}", serde_json::to_string(data).unwrap_or_default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors() {
        assert!(colors::red("error").contains("\x1b[31m"));
        assert!(colors::green("success").contains("\x1b[32m"));
        assert!(colors::yellow("warning").contains("\x1b[33m"));
        assert!(colors::bold("text").contains("\x1b[1m"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_table_row() {
        let cols = &["Name", "Status", "Value"];
        let widths = &[10, 10, 10];
        let row = table_row(cols, widths);
        assert!(row.contains("Name"));
        assert!(row.contains("Status"));
        assert!(row.contains("Value"));
    }
}
