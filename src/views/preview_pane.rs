use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tea::Model;

/// プレビューペインをレンダリング
#[allow(clippy::vec_init_then_push)]
pub fn render_preview_pane(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(preview) = &model.preview_session {
        let mut lines: Vec<Line> = Vec::new();

        // プロジェクト名
        lines.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &preview.project_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // 日時
        lines.push(Line::from(vec![
            Span::styled("Date:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(&preview.formatted_time, Style::default().fg(Color::White)),
        ]));

        // 空行
        lines.push(Line::from(""));

        // セパレータ
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(area.width.saturating_sub(2) as usize),
            Style::default().fg(Color::DarkGray),
        )]));

        lines.push(Line::from(""));

        // 最初のメッセージ
        if let Some(first_msg) = &preview.first_message {
            lines.push(Line::from(vec![Span::styled(
                "First message:",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]));

            lines.push(Line::from(""));

            // メッセージを複数行に分割
            for line in first_msg.lines().take(10) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }

            // メッセージが長い場合は省略記号を表示
            let line_count = first_msg.lines().count();
            if line_count > 10 {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    format!("... ({} more lines)", line_count - 10),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        let content = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(content, area);
    } else {
        // プレビューなし
        let empty = Paragraph::new("No session selected")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
    }
}
