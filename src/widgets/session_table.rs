use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::tea::{SessionListItem, SessionSource};

/// セッションテーブルの状態
#[derive(Debug, Default)]
pub struct SessionTableState {
    /// 選択中のインデックス
    pub selected: usize,
    /// オフセット（スクロール位置）
    pub offset: usize,
}

impl SessionTableState {
    /// 新規作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 選択を設定
    pub fn select(&mut self, index: usize) {
        self.selected = index;
    }
}

/// セッション一覧テーブルウィジェット
pub struct SessionTable<'a> {
    /// セッション一覧
    sessions: &'a [SessionListItem],
    /// ブロック（ボーダー）
    block: Option<Block<'a>>,
    /// 通常行のスタイル
    style: Style,
    /// 選択行のスタイル
    highlight_style: Style,
}

impl<'a> SessionTable<'a> {
    /// 新規作成
    pub fn new(sessions: &'a [SessionListItem]) -> Self {
        Self {
            sessions,
            block: None,
            style: Style::default(),
            highlight_style: Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// ブロックを設定
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// 通常行のスタイルを設定
    #[allow(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 選択行のスタイルを設定
    #[allow(dead_code)]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl StatefulWidget for SessionTable<'_> {
    type State = SessionTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // ブロックを描画し、内部領域を取得
        let inner_area = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner_area.height == 0 || self.sessions.is_empty() {
            return;
        }

        let visible_height = inner_area.height as usize;

        // オフセット調整（選択が見えるように）
        if state.selected < state.offset {
            state.offset = state.selected;
        } else if state.selected >= state.offset + visible_height {
            state.offset = state.selected - visible_height + 1;
        }

        // 表示する行の範囲
        let start = state.offset;
        let end = (start + visible_height).min(self.sessions.len());

        for (i, session) in self.sessions[start..end].iter().enumerate() {
            let y = inner_area.y + i as u16;
            let actual_index = start + i;
            let is_selected = actual_index == state.selected;

            let style = if is_selected {
                self.highlight_style
            } else {
                self.style
            };

            // 行を構築
            let line = self.render_session_line(session, inner_area.width as usize);

            // 行を描画
            buf.set_line(inner_area.x, y, &line, inner_area.width);

            // スタイルを適用
            for x in inner_area.x..inner_area.x + inner_area.width {
                buf[(x, y)].set_style(style);
            }
        }
    }
}

impl SessionTable<'_> {
    /// セッション行をレンダリング
    fn render_session_line(&self, session: &SessionListItem, width: usize) -> Line<'static> {
        // 時刻（固定幅）
        let time_width = 16;
        let time = format!("{:width$}", session.formatted_time, width = time_width);

        let (label_text, label_style) = match session.source {
            SessionSource::Claude => (
                "[Claude]",
                Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
            ),
            SessionSource::Codex => (
                "[Codex]",
                Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
            ),
        };
        let label_width = 8;
        let label = format!("{:width$}", label_text, width = label_width);

        // プロジェクト名（固定幅）
        let project_width = 20;
        let project = if session.project_name.len() > project_width {
            format!("{}…", &session.project_name[..project_width - 1])
        } else {
            format!("{:width$}", session.project_name, width = project_width)
        };

        // 表示テキスト（残りの幅）
        let separator = " │ ";
        let used_width =
            time_width + separator.len() + label_width + separator.len() + project_width + separator.len();
        let display_width = width.saturating_sub(used_width);
        let display = if session.latest_user_message.len() > display_width {
            if display_width > 1 {
                format!(
                    "{}…",
                    &session.latest_user_message[..display_width - 1]
                )
            } else {
                "…".to_string()
            }
        } else {
            session.latest_user_message.clone()
        };

        Line::from(vec![
            Span::raw(time),
            Span::styled(separator, Style::default().fg(Color::DarkGray)),
            Span::styled(label, label_style),
            Span::styled(separator, Style::default().fg(Color::DarkGray)),
            Span::raw(project),
            Span::styled(separator, Style::default().fg(Color::DarkGray)),
            Span::raw(display),
        ])
    }
}
