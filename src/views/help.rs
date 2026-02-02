use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tea::Model;

/// キーバインド一覧
const KEYBINDINGS: &[(&str, &str, &str)] = &[
    // Navigation
    ("j / Down", "Move down / Scroll down", "List/Detail"),
    ("k / Up", "Move up / Scroll up", "List/Detail"),
    ("Enter", "View detail", "List"),
    ("Esc", "Back / Clear filter", "All"),
    // Search & Filter
    ("/", "Search mode", "List"),
    ("f", "Filter panel", "List"),
    ("Tab", "Switch field", "Filter"),
    ("c", "Clear filter", "Filter"),
    // Detail actions
    ("y", "Copy message", "Detail"),
    ("Y", "Copy message with meta", "Detail"),
    ("u", "Toggle currency", "List/Detail"),
    // Export
    ("e", "Export session", "List/Detail"),
    // Other
    ("?", "Show help", "All"),
    ("q", "Quit", "All"),
    ("Ctrl+C", "Force quit", "All"),
];

/// ヘルプ画面をレンダリング
pub fn render_help(frame: &mut Frame, _model: &Model) {
    let area = frame.area();

    // 中央にポップアップとして表示
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = (KEYBINDINGS.len() as u16 + 6).min(area.height.saturating_sub(4));

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // 背景をクリア
    frame.render_widget(Clear, popup_area);

    // ポップアップブロック
    let block = Block::default()
        .title(" Help - Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // 内部レイアウト
    let layout = Layout::vertical([
        Constraint::Length(1), // ヘッダー行
        Constraint::Min(1),    // キーバインド一覧
        Constraint::Length(1), // フッター
    ])
    .split(inner);

    // ヘッダー
    let header = Line::from(vec![
        Span::styled(
            format!("{:<15}", "Key"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<25}", "Action"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Context",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(header), layout[0]);

    // キーバインド一覧
    let keybind_lines: Vec<Line> = KEYBINDINGS
        .iter()
        .map(|(key, action, context)| {
            Line::from(vec![
                Span::styled(format!("{:<15}", key), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<25}", action), Style::default().fg(Color::White)),
                Span::styled(*context, Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(keybind_lines), layout[1]);

    // フッター
    let footer = Line::from(vec![Span::styled(
        "Press Esc or ? to close",
        Style::default().fg(Color::DarkGray),
    )]);
    frame.render_widget(Paragraph::new(footer), layout[2]);
}
