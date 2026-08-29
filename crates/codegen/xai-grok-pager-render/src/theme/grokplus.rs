//! Grok Plus theme — GrokNight chrome with the Codex Plus Markdown palette.

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub(crate) const MD_STRONG: Color = rgb(255, 190, 175); // #ffbeaf
pub(crate) const MD_EMPHASIS: Color = rgb(247, 213, 189); // #f7d5bd
pub(crate) const MD_QUOTE_TEXT: Color = rgb(150, 127, 84); // #967f54, subdued quote text
pub(crate) const MD_QUOTE_BAR: Color = rgb(229, 192, 123); // #e5c07b, original quote bar
pub(crate) const MD_LIST: Color = rgb(108, 108, 108); // #6c6c6c
pub(crate) const MD_RULE: Color = rgb(104, 103, 112); // #686770

impl Theme {
    /// GrokNight application chrome with the Codex Plus Markdown colors.
    pub const fn grok_plus() -> Self {
        let mut theme = Self::groknight();

        theme.bg_base = Color::Reset;
        theme.bg_terminal = Color::Reset;
        theme.md_text = rgb(232, 230, 236); // #e8e6ec
        theme.md_heading_h1 = rgb(0, 204, 164); // #00cca4
        theme.md_heading_h2 = rgb(118, 175, 255); // #76afff
        theme.md_heading_h3 = rgb(174, 130, 237); // #ae82ed
        theme.md_heading_h4 = rgb(126, 126, 126); // #7e7e7e
        theme.md_heading_h5 = rgb(113, 113, 113); // #717171
        theme.md_heading_h6 = rgb(92, 92, 92); // #5c5c5c
        theme.md_heading_h1_mod = Modifier::BOLD;
        theme.md_heading_h2_mod = Modifier::BOLD;
        theme.md_heading_h3_mod = Modifier::BOLD;
        theme.md_heading_h4_mod = Modifier::BOLD;
        theme.md_heading_h5_mod = Modifier::BOLD;
        theme.md_heading_h6_mod = Modifier::BOLD;
        theme.md_code = rgb(138, 180, 248); // #8ab4f8, inline code
        theme.md_task_checked = theme.md_heading_h1;
        theme.md_task_unchecked = theme.md_text;
        theme.md_muted = MD_RULE;
        theme.link_fg = rgb(138, 180, 248); // #8ab4f8

        theme
    }
}
