//! The `[claude_compat] imported = true` marker in the user `config.toml`.
//! Once it is set, runtime fallbacks stop reading `.claude/`.

use std::path::Path;

/// Whether the user imported their Claude settings into Grok (`[claude_compat] imported`).
/// When imported, Grok skips Claude hook sources and Claude-owned MCP configs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeImport {
    Imported,
    NotImported,
}

impl ClaudeImport {
    #[must_use]
    pub fn from_marker(is_marked: bool) -> ClaudeImport {
        if is_marked {
            ClaudeImport::Imported
        } else {
            ClaudeImport::NotImported
        }
    }
}

/// Whether `<grok_home>/config.toml` sets `[claude_compat] imported = true`.
/// A missing file, a missing key, or a file that does not parse reads as not imported.
pub fn is_claude_import_marked(grok_home: &Path) -> bool {
    let Ok(contents) = std::fs::read_to_string(grok_home.join(crate::USER_CONFIG_FILENAME)) else {
        return false;
    };
    let Ok(config) = toml::from_str::<toml::Value>(&contents) else {
        return false;
    };
    config
        .get("claude_compat")
        .and_then(|section| section.get("imported"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "claude_import_tests.rs"]
mod tests;
