use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::theme::Theme;

/// 検索バーウィジェット
pub struct SearchBar<'a> {
    /// 検索クエリ
    query: &'a str,
    /// カーソル表示
    cursor_visible: bool,
    /// ブロック
    block: Option<Block<'a>>,
    /// テーマ
    theme: Theme,
}

impl<'a> SearchBar<'a> {
    /// 新規作成
    pub fn new(query: &'a str, theme: Theme) -> Self {
        Self {
            query,
            cursor_visible: true,
            block: None,
            theme,
        }
    }

    /// カーソル表示を設定
    pub fn cursor_visible(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    /// ブロックを設定
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for SearchBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let palette = self.theme.palette;
        buf.set_style(
            area,
            Style::default()
                .bg(palette.input_bg)
                .fg(palette.input_fg),
        );

        // 検索プロンプト
        let prompt = Span::styled(
            " / ",
            Style::default()
                .fg(palette.badge_fg)
                .bg(palette.badge_bg)
                .add_modifier(Modifier::BOLD),
        );

        // 検索テキスト
        let text = Span::styled(self.query, Style::default().fg(palette.input_fg));

        // カーソル
        let cursor = if self.cursor_visible {
            Span::styled(
                "_",
                Style::default()
                    .fg(palette.cursor)
                    .add_modifier(Modifier::SLOW_BLINK),
            )
        } else {
            Span::raw("")
        };

        let line = Line::from(vec![prompt, Span::raw(" "), text, cursor]);
        let mut paragraph = Paragraph::new(line);

        if let Some(block) = self.block {
            paragraph = paragraph.block(block);
        }

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn test_search_bar_new() {
        let search_bar = SearchBar::new("test query", Theme::default());
        assert_eq!(search_bar.query, "test query");
        assert!(search_bar.cursor_visible);
    }

    #[test]
    fn test_search_bar_cursor_visible() {
        let search_bar = SearchBar::new("test", Theme::default()).cursor_visible(false);
        assert!(!search_bar.cursor_visible);
    }

    #[test]
    fn test_search_bar_block() {
        let block = Block::default();
        let search_bar = SearchBar::new("test", Theme::default()).block(block);
        assert!(search_bar.block.is_some());
    }
}
