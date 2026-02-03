use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::domain::billing::format_tokens;
use crate::tea::Model;
use crate::widgets::{LineHighlight, MessageBlock, StatusBar};

/// セッション詳細ビューをレンダリング
pub fn render_session_detail(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let palette = model.theme.palette;

    let base = Block::default().style(Style::default().bg(palette.bg));
    frame.render_widget(base, area);

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
    let palette = model.theme.palette;
    let title = if let Some(session) = &model.current_session {
        let message_count = session.message_count();
        let usage_summary = session.usage_summary();
        let cost_summary = session.cost_summary();
        let tokens_value = if usage_summary.has_data {
            let suffix = if usage_summary.has_unknown { "+" } else { "" };
            Some(format!(
                "{}{}",
                format_tokens(usage_summary.total_tokens),
                suffix
            ))
        } else {
            None
        };
        let (cost_value, cost_is_na) = if cost_summary.has_data {
            if cost_summary.usd == 0.0 && cost_summary.has_unknown {
                (Some("n/a".to_string()), true)
            } else {
                let suffix = if cost_summary.has_unknown { "+" } else { "" };
                (
                    Some(format!(
                        "{}{}",
                        model.currency.format_cost(cost_summary.usd),
                        suffix
                    )),
                    false,
                )
            }
        } else {
            (None, false)
        };

        let mut meta_spans = Vec::new();
        let push_sep = |spans: &mut Vec<Span>| {
            spans.push(Span::styled(" | ", Style::default().fg(palette.text_dim)));
        };

        if let Some(tokens_value) = tokens_value {
            push_sep(&mut meta_spans);
            meta_spans.push(Span::styled(
                "Tokens: ",
                Style::default().fg(palette.text_muted),
            ));
            meta_spans.push(Span::styled(
                tokens_value,
                Style::default()
                    .fg(palette.accent_alt)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if let Some(cost_value) = cost_value {
            push_sep(&mut meta_spans);
            meta_spans.push(Span::styled(
                "Cost: ",
                Style::default().fg(palette.text_muted),
            ));
            let cost_style = if cost_is_na {
                Style::default().fg(palette.text_dim)
            } else {
                Style::default()
                    .fg(palette.warning)
                    .add_modifier(Modifier::BOLD)
            };
            meta_spans.push(Span::styled(cost_value, cost_style));
        }

        push_sep(&mut meta_spans);
        meta_spans.push(Span::styled(
            model.currency.label().to_string(),
            Style::default().fg(palette.text_dim),
        ));

        let mut spans = Vec::new();
        spans.push(Span::styled(
            " katha ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("- ", Style::default().fg(palette.text_dim)));
        spans.push(Span::styled(
            session.project_name(),
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(" (", Style::default().fg(palette.text_muted)));
        spans.push(Span::styled(
            format!("{}", message_count),
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            " messages)",
            Style::default().fg(palette.text_muted),
        ));
        spans.extend(meta_spans);
        Line::from(spans)
    } else if let Some(selected) = model.selected_session() {
        Line::from(vec![
            Span::styled(
                " katha ",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("- ", Style::default().fg(palette.text_dim)),
            Span::styled(
                &selected.project_name,
                Style::default()
                    .fg(palette.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![Span::styled(
            " katha ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )])
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.surface));

    let header = Paragraph::new(title)
        .block(block)
        .style(Style::default().fg(palette.text).bg(palette.surface));
    frame.render_widget(header, area);
}

/// コンテンツ領域をレンダリング
fn render_content(frame: &mut Frame, area: Rect, model: &Model) {
    let palette = model.theme.palette;
    let block = Block::default()
        .title(" Messages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    if model.current_session.is_some() {
        // セッションが読み込まれている場合
        let messages = model.detail_message_entries();

        if messages.is_empty() {
            let empty = Paragraph::new("No messages found")
                .style(Style::default().fg(palette.text_dim))
                .block(block);
            frame.render_widget(empty, area);
            return;
        }

        // 内側の幅を計算（ボーダー分を除く）
        let inner_width = area.width.saturating_sub(2);

        // メッセージをテキストに変換
        let mut lines: Vec<Line> = Vec::new();

        for entry in messages.iter() {
            let message_block =
                MessageBlock::new(*entry, inner_width, model.currency, model.theme);
            let block_lines = message_block.to_lines();
            lines.extend(block_lines);
        }

        let total_lines = model.detail_total_lines();
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
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.bg));
        let inner_area = block.inner(area);

        let content = Paragraph::new(Text::from(lines))
            .block(block)
            .style(Style::default().fg(palette.text).bg(palette.bg))
            .wrap(Wrap { trim: false })
            .scroll((model.detail_scroll_offset as u16, 0));
        frame.render_widget(content, area);

        if visible_height > 0 {
            let cursor_row = model
                .detail_cursor_row
                .min(visible_height.saturating_sub(1)) as u16;
            let highlight_style = Style::default()
                .bg(palette.selection_bg)
                .fg(palette.selection_fg)
                .add_modifier(Modifier::BOLD);
            frame.render_widget(LineHighlight::new(cursor_row, highlight_style), inner_area);
        }
    } else {
        // 読み込み中
        let loading = Paragraph::new("Loading session...")
            .style(Style::default().fg(palette.warning))
            .block(block);
        frame.render_widget(loading, area);
    }
}

/// フッターをレンダリング（ステータスバー + キーバインド）
fn render_footer(frame: &mut Frame, area: Rect, model: &Model) {
    let palette = model.theme.palette;
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

        let keybind_bg = Block::default().style(Style::default().bg(palette.surface));
        frame.render_widget(keybind_bg, keybind_area);

        let keys = [
            ("Esc/q", "Back"),
            ("j/↓", "Down"),
            ("k/↑", "Up"),
            ("y", "Copy"),
            ("Y", "Copy+Meta"),
            ("u", "Currency"),
            ("Ctrl+t", "Theme"),
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
                            .fg(palette.badge_fg)
                            .bg(palette.badge_bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {} ", action),
                        Style::default().fg(palette.text_dim),
                    ),
                    Span::raw(" "),
                ]
            })
            .collect();

        let keybinds = Paragraph::new(Line::from(spans))
            .style(Style::default().fg(palette.text).bg(palette.surface));
        frame.render_widget(keybinds, keybind_area);
    }
}
