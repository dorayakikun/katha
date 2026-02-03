use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::tea::{SessionSource, TreeItem, TreeNodeKind};
use crate::theme::Theme;

/// 文字列を指定した文字数で安全に切り詰める（UTF-8 対応）
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else if max_chars > 1 {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    } else {
        "…".to_string()
    }
}

/// プロジェクトツリーの状態
#[derive(Debug, Default)]
pub struct ProjectTreeState {
    /// 選択中のインデックス
    pub selected: usize,
    /// オフセット（スクロール位置）
    pub offset: usize,
}

impl ProjectTreeState {
    /// 新規作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 選択を設定
    pub fn select(&mut self, index: usize) {
        self.selected = index;
    }
}

/// プロジェクトツリーウィジェット
pub struct ProjectTree<'a> {
    /// ツリーアイテム一覧
    items: &'a [TreeItem],
    /// 展開されているプロジェクトのパス
    expanded: &'a HashSet<String>,
    /// ブロック（ボーダー）
    block: Option<Block<'a>>,
    /// 通常行のスタイル
    style: Style,
    /// 選択行のスタイル
    highlight_style: Style,
    /// テーマ
    theme: Theme,
}

impl<'a> ProjectTree<'a> {
    /// 新規作成
    pub fn new(items: &'a [TreeItem], expanded: &'a HashSet<String>, theme: Theme) -> Self {
        let palette = theme.palette;
        Self {
            items,
            expanded,
            block: None,
            style: Style::default(),
            highlight_style: Style::default()
                .bg(palette.selection_bg)
                .fg(palette.selection_fg)
                .add_modifier(Modifier::BOLD),
            theme,
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

impl StatefulWidget for ProjectTree<'_> {
    type State = ProjectTreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // ブロックを描画し、内部領域を取得
        let inner_area = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner_area.height == 0 || self.items.is_empty() {
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
        let end = (start + visible_height).min(self.items.len());

        for (i, item) in self.items[start..end].iter().enumerate() {
            let y = inner_area.y + i as u16;
            let actual_index = start + i;
            let is_selected = actual_index == state.selected;

            let style = if is_selected {
                self.highlight_style
            } else {
                self.style
            };

            // 行を構築
            let line = self.render_tree_line(item, inner_area.width as usize);

            // 行を描画
            buf.set_line(inner_area.x, y, &line, inner_area.width);

            // スタイルを適用
            for x in inner_area.x..inner_area.x + inner_area.width {
                buf[(x, y)].set_style(style);
            }
        }
    }
}

impl ProjectTree<'_> {
    /// ツリー行をレンダリング
    fn render_tree_line(&self, item: &TreeItem, width: usize) -> Line<'static> {
        match item.kind {
            TreeNodeKind::Project => self.render_project_line(item, width),
            TreeNodeKind::Session => self.render_session_line(item, width),
        }
    }

    /// プロジェクト行をレンダリング
    fn render_project_line(&self, item: &TreeItem, width: usize) -> Line<'static> {
        let palette = self.theme.palette;
        let is_expanded = self.expanded.contains(&item.project_path);
        let arrow = if is_expanded { "▼ " } else { "▶ " };

        // セッション数
        let count_str = format!(" ({} sessions)", item.child_count);

        // 最新時刻
        let time_str = format!(" │ {}", item.formatted_time);

        // プロジェクト名の幅を計算（文字単位）
        let fixed_width = arrow.chars().count() + count_str.chars().count() + time_str.chars().count();
        let name_width = width.saturating_sub(fixed_width);

        // UTF-8 安全な切り詰め
        let project_name = truncate_str(&item.project_name, name_width);

        Line::from(vec![
            Span::styled(arrow, Style::default().fg(palette.accent)),
            Span::styled(
                project_name,
                Style::default()
                    .fg(palette.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(count_str, Style::default().fg(palette.text_dim)),
            Span::styled(time_str, Style::default().fg(palette.text_dim)),
        ])
    }

    /// セッション行をレンダリング
    fn render_session_line(&self, item: &TreeItem, width: usize) -> Line<'static> {
        let palette = self.theme.palette;
        let indent = "    ";
        let separator = " │ ";
        let label_separator = " ";

        // 時刻
        let time_str = &item.formatted_time;

        // メッセージ表示
        let display = item
            .session
            .as_ref()
            .map(|s| s.latest_user_message.as_str())
            .unwrap_or("");

        let (label_text, label_style) = item
            .session
            .as_ref()
            .map(|s| match s.source {
                SessionSource::Claude => (
                    "[Claude]",
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                SessionSource::Codex => (
                    "[Codex]",
                    Style::default()
                        .fg(palette.success)
                        .add_modifier(Modifier::BOLD),
                ),
            })
            .unwrap_or(("[Unknown]", Style::default().fg(palette.text_dim)));
        let label_width = 8;
        let label = format!("{:width$}", label_text, width = label_width);

        // 表示幅を計算（文字単位）
        let fixed_width = indent.chars().count()
            + time_str.chars().count()
            + label_separator.chars().count()
            + label_width
            + separator.chars().count();
        let display_width = width.saturating_sub(fixed_width);

        // UTF-8 安全な切り詰め
        let display_text = truncate_str(display, display_width);

        Line::from(vec![
            Span::styled(indent, Style::default().fg(palette.text_dim)),
            Span::styled(time_str.to_string(), Style::default().fg(palette.text_dim)),
            Span::styled(label_separator, Style::default().fg(palette.text_dim)),
            Span::styled(label, label_style),
            Span::styled(separator, Style::default().fg(palette.text_dim)),
            Span::styled(display_text, Style::default().fg(palette.text)),
        ])
    }
}
