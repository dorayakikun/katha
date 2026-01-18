use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tea::Model;
use crate::widgets::{MessageBlock, StatusBar};

/// セッション詳細ビューをレンダリング
pub fn render_session_detail(frame: &mut Frame, model: &Model) {
    let area = frame.area();

    // 3分割レイアウト: ヘッダー(3行) | メイン(可変) | フッター(3行)
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(area);

    render_header(frame, layout[0], model);
    render_content(frame, layout[1], model);
    render_footer(frame, layout[2], model);
}

/// ヘッダーをレンダリング
fn render_header(frame: &mut Frame, area: Rect, model: &Model) {
    let title = if let Some(session) = &model.current_session {
        let message_count = session.message_count();
        Line::from(vec![
            Span::styled(
                " katha ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("- ", Style::default().fg(Color::Gray)),
            Span::styled(
                session.project_name(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({} messages)", message_count),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else if let Some(selected) = model.selected_session() {
        Line::from(vec![
            Span::styled(
                " katha ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("- ", Style::default().fg(Color::Gray)),
            Span::styled(
                &selected.project_name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![Span::styled(
            " katha ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let header = Paragraph::new(title).block(block);
    frame.render_widget(header, area);
}

/// コンテンツ領域をレンダリング
fn render_content(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::default()
        .title(" Messages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(session) = &model.current_session {
        // セッションが読み込まれている場合
        let messages: Vec<_> = session
            .entries
            .iter()
            .filter(|e| e.is_user() || e.is_assistant())
            .filter(|e| e.display_text().is_some()) // テキストがあるメッセージのみ表示
            .collect();

        if messages.is_empty() {
            let empty = Paragraph::new("No messages found")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(empty, area);
            return;
        }

        // 内側の幅を計算（ボーダー分を除く）
        let inner_width = area.width.saturating_sub(2);

        // メッセージをテキストに変換
        let mut lines: Vec<Line> = Vec::new();

        for entry in messages {
            let message_block = MessageBlock::new(entry, inner_width);
            lines.extend(message_block.to_lines());
        }

        let total_lines = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize; // ボーダー分を除く

        // スクロールインジケーターを表示
        let scroll_info = if total_lines > visible_height {
            let max_scroll = total_lines.saturating_sub(visible_height);
            let scroll_pos = model.detail_scroll_offset.min(max_scroll);
            let percent = if max_scroll > 0 {
                (scroll_pos * 100) / max_scroll
            } else {
                0
            };
            format!(" [{}/{}] {}% ", scroll_pos + 1, total_lines, percent)
        } else {
            String::new()
        };

        let block = Block::default()
            .title(format!(" Messages{}", scroll_info))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let content = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((model.detail_scroll_offset as u16, 0));
        frame.render_widget(content, area);
    } else {
        // 読み込み中
        let loading = Paragraph::new("Loading session...")
            .style(Style::default().fg(Color::Yellow))
            .block(block);
        frame.render_widget(loading, area);
    }
}

/// フッターをレンダリング（ステータスバー + キーバインド）
fn render_footer(frame: &mut Frame, area: Rect, model: &Model) {
    // ステータスバーを描画
    let status_bar = StatusBar::new(model);
    frame.render_widget(status_bar, area);

    // キーバインドを描画（ステータスバーの下に配置）
    if area.height > 1 {
        let keybind_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(1),
        };

        let keys = [
            ("Esc/q", "Back"),
            ("j/↓", "Down"),
            ("k/↑", "Up"),
            ("e", "Export"),
            ("?", "Help"),
        ];

        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, action)| {
                vec![
                    Span::styled(
                        format!(" {} ", key),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {} ", action), Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                ]
            })
            .collect();

        let keybinds = Paragraph::new(Line::from(spans));
        frame.render_widget(keybinds, keybind_area);
    }
}
