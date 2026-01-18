use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::domain::SessionEntry;

/// メッセージブロックのスタイル定義
pub struct MessageStyles;

impl MessageStyles {
    /// ユーザーロールのスタイル
    pub fn user_role() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// アシスタントロールのスタイル
    pub fn assistant_role() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }

    /// セパレータのスタイル
    pub fn separator() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// タイムスタンプのスタイル
    pub fn timestamp() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// ツール呼び出しのスタイル
    pub fn tool_use() -> Style {
        Style::default().fg(Color::Yellow)
    }
}

/// メッセージブロックウィジェット
pub struct MessageBlock<'a> {
    entry: &'a SessionEntry,
    width: u16,
}

impl<'a> MessageBlock<'a> {
    /// 新規作成
    pub fn new(entry: &'a SessionEntry, width: u16) -> Self {
        Self { entry, width }
    }

    /// メッセージをレンダリング用の行に変換
    pub fn to_lines(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        // ロールヘッダー
        lines.push(self.render_header());

        // メッセージ本文
        if let Some(text) = self.entry.display_text() {
            for line in text.lines() {
                lines.push(Line::from(Span::raw(line.to_string())));
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
        let (role_label, role_style) = if self.entry.is_user() {
            ("user", MessageStyles::user_role())
        } else {
            ("assistant", MessageStyles::assistant_role())
        };

        let timestamp = self
            .entry
            .datetime()
            .map(|dt| dt.format(" %H:%M:%S").to_string())
            .unwrap_or_default();

        let separator_len = self
            .width
            .saturating_sub(role_label.len() as u16 + 4 + timestamp.len() as u16);

        Line::from(vec![
            Span::styled(format!("── {} ", role_label), role_style),
            Span::styled(
                "─".repeat(separator_len as usize),
                MessageStyles::separator(),
            ),
            Span::styled(timestamp, MessageStyles::timestamp()),
        ])
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

        for (_id, name, _input) in tool_uses {
            lines.push(Line::from(vec![
                Span::styled("  ⚙ ", MessageStyles::tool_use()),
                Span::styled(name.to_string(), MessageStyles::tool_use()),
            ]));
        }

        Some(lines)
    }
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

    fn create_test_entry(entry_type: &str) -> SessionEntry {
        SessionEntry {
            entry_type: entry_type.to_string(),
            timestamp: Some("2025-01-01T12:00:00Z".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_message_styles() {
        let user = MessageStyles::user_role();
        assert_eq!(user.fg, Some(Color::Cyan));

        let assistant = MessageStyles::assistant_role();
        assert_eq!(assistant.fg, Some(Color::Green));
    }

    #[test]
    fn test_message_block_user() {
        let entry = create_test_entry("user");
        let block = MessageBlock::new(&entry, 80);
        let lines = block.to_lines();

        assert!(!lines.is_empty());
    }

    #[test]
    fn test_message_block_assistant() {
        let entry = create_test_entry("assistant");
        let block = MessageBlock::new(&entry, 80);
        let lines = block.to_lines();

        assert!(!lines.is_empty());
    }
}
