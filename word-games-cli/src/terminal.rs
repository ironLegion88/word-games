//! Terminal styling, color management, and plain-text fallbacks.
//!
//! Adapted from Mastermind `src/ui.rs` (MM-22).

use std::fmt;

/// Color formatting mode for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// ANSI escape sequences enabled for rich colors.
    Always,
    /// Plain-text formatting with zero ANSI escape sequences.
    Never,
    /// Automatically detects color capability based on NO_COLOR and terminal output.
    #[default]
    Auto,
}

impl ColorMode {
    /// Resolves `Auto` into an active boolean: `true` if ANSI should be emitted.
    pub fn is_active(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => {
                if std::env::var_os("NO_COLOR").is_some() {
                    false
                } else {
                    // Default to true in interactive terminals; could probe isatty if desired
                    true
                }
            }
        }
    }

    /// Wraps text with an ANSI escape code, or returns plain text if disabled.
    pub fn style(&self, code: &str, text: &str) -> String {
        if self.is_active() {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    pub fn bold(&self, text: &str) -> String {
        self.style("1", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.style("2", text)
    }

    pub fn red(&self, text: &str) -> String {
        self.style("31", text)
    }

    pub fn green(&self, text: &str) -> String {
        self.style("32", text)
    }

    pub fn yellow(&self, text: &str) -> String {
        self.style("33", text)
    }

    pub fn blue(&self, text: &str) -> String {
        self.style("34", text)
    }

    pub fn magenta(&self, text: &str) -> String {
        self.style("35", text)
    }

    pub fn cyan(&self, text: &str) -> String {
        self.style("36", text)
    }

    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "always" | "on" | "true" => Some(Self::Always),
            "never" | "off" | "false" | "plain" => Some(Self::Never),
            _ => None,
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
            Self::Auto => write!(f, "auto"),
        }
    }
}
