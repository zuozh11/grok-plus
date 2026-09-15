//! Grok Plus theme — terminal-native chrome with the Codex Plus Markdown palette.

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub(crate) const MD_STRONG: Color = rgb(255, 190, 175); // #ffbeaf
pub(crate) const MD_EMPHASIS: Color = rgb(247, 213, 189); // #f7d5bd
pub(crate) const MD_LIST: Color = rgb(108, 108, 108); // #6c6c6c
pub(crate) const MD_RULE: Color = rgb(104, 103, 112); // #686770

impl Theme {
    /// Terminal-native chrome ([`Theme::terminal`]) with the Codex Plus Markdown colors.
    ///
    /// Every surface stays on the terminal canvas and borrows the profile's
    /// ANSI palette, so only the Markdown palette is fixed RGB: it assumes a
    /// dark profile and does not follow a light one.
    pub const fn grok_plus() -> Self {
        let mut theme = Self::terminal();

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The chrome is `Theme::terminal`: zero opaque cells, reverse-video
    /// selection, bright-black decoration. Only Markdown is fixed RGB.
    #[test]
    fn grok_plus_keeps_terminal_chrome() {
        let theme = Theme::grok_plus();
        for (name, color) in [
            ("bg_base", theme.bg_base),
            ("bg_light", theme.bg_light),
            ("bg_dark", theme.bg_dark),
            ("bg_highlight", theme.bg_highlight),
            ("bg_hover", theme.bg_hover),
            ("bg_terminal", theme.bg_terminal),
            ("bg_visual", theme.bg_visual),
            ("md_code_bg", theme.md_code_bg),
            ("text_primary", theme.text_primary),
            ("accent_user", theme.accent_user),
        ] {
            assert_eq!(color, Color::Reset, "{name} must defer to the canvas");
        }
        assert!(theme.is_bandless());
        assert_eq!(theme.prompt_border, Color::DarkGray);
        assert!(
            theme
                .selection_overlay()
                .add_modifier
                .contains(Modifier::REVERSED)
        );
    }

    #[test]
    fn grok_plus_paints_codex_markdown() {
        let theme = Theme::grok_plus();
        assert_eq!(theme.md_text, rgb(232, 230, 236));
        assert_eq!(theme.md_heading_h1, rgb(0, 204, 164));
        assert_eq!(theme.md_heading_h2, rgb(118, 175, 255));
        assert_eq!(theme.md_heading_h3, rgb(174, 130, 237));
        assert_eq!(theme.md_heading_h4, rgb(126, 126, 126));
        assert_eq!(theme.md_heading_h5, rgb(113, 113, 113));
        assert_eq!(theme.md_heading_h6, rgb(92, 92, 92));
        assert_eq!(theme.md_code, rgb(138, 180, 248));
        assert_eq!(theme.link_fg, rgb(138, 180, 248));
        assert_eq!(theme.md_task_checked, theme.md_heading_h1);
        assert_eq!(theme.md_task_unchecked, theme.md_text);
        assert_eq!(theme.md_muted, MD_RULE);
        for (name, modifier) in [
            ("h1", theme.md_heading_h1_mod),
            ("h2", theme.md_heading_h2_mod),
            ("h3", theme.md_heading_h3_mod),
            ("h4", theme.md_heading_h4_mod),
            ("h5", theme.md_heading_h5_mod),
            ("h6", theme.md_heading_h6_mod),
        ] {
            assert_eq!(modifier, Modifier::BOLD, "{name} stays bold");
        }
    }
}
