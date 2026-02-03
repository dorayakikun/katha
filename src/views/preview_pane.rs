use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tea::Model;

/// プレビューペインをレンダリング
#[allow(clippy::vec_init_then_push)]
pub fn render_preview_pane(frame: &mut Frame, area: Rect, model: &Model) {
    let palette = model.theme.palette;
    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    if let Some(preview) = &model.preview_session {
        let mut lines: Vec<Line> = Vec::new();

        // プロジェクト名
        lines.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(palette.text_muted)),
            Span::styled(
                &preview.project_name,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // 日時
        lines.push(Line::from(vec![
            Span::styled("Date:    ", Style::default().fg(palette.text_muted)),
            Span::styled(&preview.formatted_time, Style::default().fg(palette.text)),
        ]));

        // 空行
        lines.push(Line::from(""));

        // セパレータ
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(area.width.saturating_sub(2) as usize),
            Style::default().fg(palette.text_dim),
        )]));

        lines.push(Line::from(""));

        // 最新のユーザーメッセージ
        if let Some(latest_msg) = &preview.latest_user_message {
            lines.push(Line::from(vec![Span::styled(
                "Latest user message:",
                Style::default()
                    .fg(palette.text_muted)
                    .add_modifier(Modifier::ITALIC),
            )]));

            lines.push(Line::from(""));

            // メッセージを複数行に分割
            for line in latest_msg.lines().take(10) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(palette.text),
                )));
            }

            // メッセージが長い場合は省略記号を表示
            let line_count = latest_msg.lines().count();
            if line_count > 10 {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    format!("... ({} more lines)", line_count - 10),
                    Style::default().fg(palette.text_dim),
                )]));
            }
        }

        let content = Paragraph::new(Text::from(lines))
            .block(block)
            .style(Style::default().fg(palette.text).bg(palette.bg))
            .wrap(Wrap { trim: false });
        frame.render_widget(content, area);
    } else {
        // プレビューなし
        let empty = Paragraph::new("No session selected")
            .style(Style::default().fg(palette.text_dim))
            .block(block);
        frame.render_widget(empty, area);
    }
}
