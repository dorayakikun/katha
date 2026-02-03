use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::domain::SessionEntry;
use crate::domain::billing::{Currency, estimate_cost_usd, format_tokens};
use crate::theme::Theme;
use unicode_width::UnicodeWidthStr;

/// メッセージブロックのスタイル定義
pub struct MessageStyles {
    palette: crate::theme::Palette,
}

impl MessageStyles {
    pub fn new(theme: Theme) -> Self {
        Self {
            palette: theme.palette,
        }
    }

    /// ユーザーロールのスタイル
    pub fn user_role(&self) -> Style {
        Style::default()
            .fg(self.palette.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// アシスタントロールのスタイル
    pub fn assistant_role(&self) -> Style {
        Style::default()
            .fg(self.palette.success)
            .add_modifier(Modifier::BOLD)
    }

    /// セパレータのスタイル
    pub fn separator(&self) -> Style {
        Style::default().fg(self.palette.text_dim)
    }

    /// タイムスタンプのスタイル
    pub fn timestamp(&self) -> Style {
        Style::default().fg(self.palette.text_dim)
    }

    /// ツール呼び出しのスタイル
    pub fn tool_use(&self) -> Style {
        Style::default().fg(self.palette.accent_alt)
    }
}

/// メッセージブロックウィジェット
pub struct MessageBlock<'a> {
    entry: &'a SessionEntry,
    width: u16,
    currency: Currency,
    theme: Theme,
}

impl<'a> MessageBlock<'a> {
    /// 新規作成
    pub fn new(entry: &'a SessionEntry, width: u16, currency: Currency, theme: Theme) -> Self {
        Self {
            entry,
            width,
            currency,
            theme,
        }
    }

    /// メッセージをレンダリング用の行に変換
    pub fn to_lines(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();
        let palette = self.theme.palette;

        // ロールヘッダー
        lines.push(self.render_header());

        // メッセージ本文
        if let Some(text) = self.entry.display_text() {
            for line in text.lines() {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(palette.text),
                )));
            }
        }

        // ツール呼び出し情報（あれば）
        if let Some(tool_lines) = self.render_tool_use() {
            lines.extend(tool_lines);
        }

        // 空行
        lines.push(Line::from(""));

        lines
    }

    /// ヘッダー行をレンダリング
    fn render_header(&self) -> Line<'a> {
        let styles = MessageStyles::new(self.theme);
        let palette = self.theme.palette;
        let (role_label, role_style) = if self.entry.is_user() {
            ("user", styles.user_role())
        } else {
            ("assistant", styles.assistant_role())
        };

        let timestamp = self
            .entry
            .datetime()
            .map(|dt| dt.format(" %H:%M:%S").to_string())
            .unwrap_or_default();

        let usage_meta = self.usage_meta();
        let meta_plain = if let Some(usage) = &usage_meta {
            format!("{timestamp} {} | {}", usage.tokens_text, usage.cost_text)
        } else {
            timestamp.clone()
        };

        let meta_width = UnicodeWidthStr::width(meta_plain.as_str()) as u16;
        let separator_len = self
            .width
            .saturating_sub(role_label.len() as u16 + 4 + meta_width);

        let mut meta_spans: Vec<Span> = Vec::new();
        meta_spans.push(Span::styled(timestamp, styles.timestamp()));
        if let Some(usage) = usage_meta {
            meta_spans.push(Span::raw(" "));
            meta_spans.push(Span::styled(
                usage.tokens_text,
                Style::default()
                    .fg(palette.accent_alt)
                    .add_modifier(Modifier::BOLD),
            ));
            meta_spans.push(Span::styled(
                " | ",
                Style::default().fg(palette.text_dim),
            ));
            let cost_style = if usage.cost_is_na {
                Style::default().fg(palette.text_dim)
            } else {
                Style::default()
                    .fg(palette.warning)
                    .add_modifier(Modifier::BOLD)
            };
            meta_spans.push(Span::styled(usage.cost_text, cost_style));
        }

        let mut spans = Vec::new();
        spans.push(Span::styled(format!("── {} ", role_label), role_style));
        spans.push(Span::styled(
            "─".repeat(separator_len as usize),
            styles.separator(),
        ));
        spans.extend(meta_spans);
        Line::from(spans)
    }

    /// ツール呼び出し情報をレンダリング
    fn render_tool_use(&self) -> Option<Vec<Line<'a>>> {
        let message = self.entry.message.as_ref()?;
        let tool_uses = message.tool_uses();

        if tool_uses.is_empty() {
            return None;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(""));
        let styles = MessageStyles::new(self.theme);

        for (_id, name, _input) in tool_uses {
            lines.push(Line::from(vec![
                Span::styled("  ⚙ ", styles.tool_use()),
                Span::styled(name.to_string(), styles.tool_use()),
            ]));
        }

        Some(lines)
    }

    fn usage_meta(&self) -> Option<UsageMeta> {
        let message = self.entry.message.as_ref()?;
        let usage = message.usage.as_ref()?;
        if usage.input_tokens.is_none() || usage.output_tokens.is_none() {
            return None;
        }
        let total_tokens = usage.total_tokens();
        let tokens_text = format!("{} tok", format_tokens(total_tokens));

        let cost_text = message
            .model
            .as_deref()
            .and_then(|model| estimate_cost_usd(model, usage))
            .map(|usd| self.currency.format_cost(usd))
            .unwrap_or_else(|| "n/a".to_string());
        let cost_is_na = cost_text == "n/a";

        Some(UsageMeta {
            tokens_text,
            cost_text,
            cost_is_na,
        })
    }
}

struct UsageMeta {
    tokens_text: String,
    cost_text: String,
    cost_is_na: bool,
}

impl Widget for MessageBlock<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.to_lines();

        for (i, line) in lines.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }

            let y = area.y + i as u16;
            buf.set_line(area.x, y, line, area.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn create_test_entry(entry_type: &str) -> SessionEntry {
        SessionEntry {
            entry_type: Some(entry_type.to_string()),
            timestamp: Some("2025-01-01T12:00:00Z".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_message_styles() {
        let theme = Theme::default();
        let styles = MessageStyles::new(theme);
        let user = styles.user_role();
        assert_eq!(user.fg, Some(theme.palette.accent));

        let assistant = styles.assistant_role();
        assert_eq!(assistant.fg, Some(theme.palette.success));
    }

    #[test]
    fn test_message_block_user() {
        let entry = create_test_entry("user");
        let block = MessageBlock::new(&entry, 80, Currency::Usd, Theme::default());
        let lines = block.to_lines();

        assert!(!lines.is_empty());
    }

    #[test]
    fn test_message_block_assistant() {
        let entry = create_test_entry("assistant");
        let block = MessageBlock::new(&entry, 80, Currency::Usd, Theme::default());
        let lines = block.to_lines();

        assert!(!lines.is_empty());
    }
}
