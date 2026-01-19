use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::tea::{Model, ViewMode};

/// ステータスバーウィジェット
/// フィルタ状態、セッション数、現在のモードを表示する
pub struct StatusBar<'a> {
    filtered_count: usize,
    total_count: usize,
    is_filtered: bool,
    view_mode: ViewMode,
    search_query: Option<&'a str>,
    project_filter: Option<&'a str>,
    error_message: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    /// Model からステータスバーを作成
    pub fn new(model: &'a Model) -> Self {
        let search_query = if model.search_query.is_empty() {
            None
        } else {
            Some(model.search_query.text.as_str())
        };

        let project_filter = model.filter_criteria.project_filter.as_deref();

        Self {
            filtered_count: model.filtered_count(),
            total_count: model.sessions.len(),
            is_filtered: model.is_filtered,
            view_mode: model.view_mode,
            search_query,
            project_filter,
            error_message: model.error_message.as_deref(),
        }
    }

    /// ビューモード表示テキストを取得
    fn mode_text(&self) -> &'static str {
        match self.view_mode {
            ViewMode::SessionList => "List",
            ViewMode::SessionDetail => "Detail",
            ViewMode::Search => "Search",
            ViewMode::Filter => "Filter",
            ViewMode::Help => "Help",
            ViewMode::Export => "Export",
        }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let mut spans: Vec<Span> = Vec::new();

        // モード表示
        spans.push(Span::styled(
            format!("[{}]", self.mode_text()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::raw(" "));

        // セッション数表示
        let count_text = if self.is_filtered {
            format!("Sessions: {}/{}", self.filtered_count, self.total_count)
        } else {
            format!("Sessions: {}", self.total_count)
        };
        spans.push(Span::styled(count_text, Style::default().fg(Color::White)));

        // フィルタ状態表示
        if let Some(project) = self.project_filter {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("Project: \"{}\"", project),
                Style::default().fg(Color::Yellow),
            ));
        }

        // 検索クエリ表示
        if let Some(query) = self.search_query {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("Search: \"{}\"", query),
                Style::default().fg(Color::Green),
            ));
        }

        if let Some(error) = self.error_message {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("Error: {}", error),
                Style::default().fg(Color::Red),
            ));
        }

        let line = Line::from(spans);

        // 1行目に表示
        if inner.height > 0 {
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_mode_text() {
        let model = Model::new();
        let status_bar = StatusBar::new(&model);
        assert_eq!(status_bar.mode_text(), "List");
    }

    #[test]
    fn test_status_bar_counts() {
        use crate::tea::SessionListItem;
        use chrono::Utc;

        let sessions: Vec<SessionListItem> = (0..10)
            .map(|i| SessionListItem {
                session_id: format!("session-{}", i),
                source: crate::tea::SessionSource::Claude,
                project_name: format!("project-{}", i),
                project_path: format!("/path/to/project-{}", i),
                latest_user_message: format!("Message {}", i),
                formatted_time: "2025-01-01 00:00".to_string(),
                datetime: Utc::now(),
            })
            .collect();

        let model = Model::new().with_sessions(sessions);
        let status_bar = StatusBar::new(&model);

        assert_eq!(status_bar.total_count, 10);
        assert_eq!(status_bar.filtered_count, 10);
        assert!(!status_bar.is_filtered);
    }
}
