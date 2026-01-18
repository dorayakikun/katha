use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

/// 検索バーウィジェット
pub struct SearchBar<'a> {
    /// 検索クエリ
    query: &'a str,
    /// カーソル表示
    cursor_visible: bool,
    /// ブロック
    block: Option<Block<'a>>,
}

impl<'a> SearchBar<'a> {
    /// 新規作成
    pub fn new(query: &'a str) -> Self {
        Self {
            query,
            cursor_visible: true,
            block: None,
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
        // 検索プロンプト
        let prompt = Span::styled(
            " / ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        // 検索テキスト
        let text = Span::styled(self.query, Style::default().fg(Color::White));

        // カーソル
        let cursor = if self.cursor_visible {
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Yellow)
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

    #[test]
    fn test_search_bar_new() {
        let search_bar = SearchBar::new("test query");
        assert_eq!(search_bar.query, "test query");
        assert!(search_bar.cursor_visible);
    }

    #[test]
    fn test_search_bar_cursor_visible() {
        let search_bar = SearchBar::new("test").cursor_visible(false);
        assert!(!search_bar.cursor_visible);
    }

    #[test]
    fn test_search_bar_block() {
        let block = Block::default();
        let search_bar = SearchBar::new("test").block(block);
        assert!(search_bar.block.is_some());
    }
}
