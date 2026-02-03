use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::export::{ExportFormat, generate_filename};
use crate::tea::{ExportStatus, Model};

/// スピナーのフレーム
const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// 現在のスピナーフレームを取得
fn spinner_frame() -> char {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let frame_index = (millis / 100) as usize % SPINNER_FRAMES.len();
    SPINNER_FRAMES[frame_index]
}

/// エクスポートダイアログをレンダリング
pub fn render_export_dialog(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let palette = model.theme.palette;

    // 中央にポップアップとして表示
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 12.min(area.height.saturating_sub(4));

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // 背景をクリア
    frame.render_widget(Clear, popup_area);

    // ポップアップブロック
    let block = Block::default()
        .title(" Export Session ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.surface));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // 内部レイアウト
    let layout = Layout::vertical([
        Constraint::Length(1), // 空行
        Constraint::Length(1), // Format ラベル
        Constraint::Length(1), // Format 選択
        Constraint::Length(1), // 空行
        Constraint::Length(1), // Output ラベル
        Constraint::Length(1), // Output パス
        Constraint::Length(1), // 空行
        Constraint::Length(1), // ステータス
        Constraint::Min(1),    // フッター
    ])
    .split(inner);

    // Format ラベル
    let format_label = Paragraph::new("Format:").style(Style::default().fg(palette.text));
    frame.render_widget(format_label, layout[1]);

    // Format 選択
    let md_style = if model.export_format == ExportFormat::Markdown {
        Style::default()
            .fg(palette.accent_alt)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.text_dim)
    };
    let json_style = if model.export_format == ExportFormat::Json {
        Style::default()
            .fg(palette.accent_alt)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.text_dim)
    };

    let md_marker = if model.export_format == ExportFormat::Markdown {
        "[*]"
    } else {
        "[ ]"
    };
    let json_marker = if model.export_format == ExportFormat::Json {
        "[*]"
    } else {
        "[ ]"
    };

    let format_line = Line::from(vec![
        Span::styled(format!("  {} Markdown  ", md_marker), md_style),
        Span::styled(format!("{} JSON", json_marker), json_style),
    ]);
    frame.render_widget(Paragraph::new(format_line), layout[2]);

    // Output ラベル
    let output_label = Paragraph::new("Output:").style(Style::default().fg(palette.text));
    frame.render_widget(output_label, layout[4]);

    // Output パス
    let filename = if let Some(session) = &model.current_session {
        generate_filename(session, model.export_format)
    } else if let Some(selected) = model.selected_session() {
        format!(
            "{}_{}.{}",
            selected.project_name,
            &selected.session_id[..8.min(selected.session_id.len())],
            model.export_format.extension()
        )
    } else {
        format!("session.{}", model.export_format.extension())
    };

    let output_path =
        Paragraph::new(format!("  ./{}", filename)).style(Style::default().fg(palette.text_dim));
    frame.render_widget(output_path, layout[5]);

    // ステータス表示
    if let Some(status) = &model.export_status {
        let status_line = match status {
            ExportStatus::Selecting => Line::from(Span::styled("", Style::default())),
            ExportStatus::Exporting => {
                let spinner = spinner_frame();
                Line::from(Span::styled(
                    format!("  {} Exporting...", spinner),
                    Style::default().fg(palette.warning),
                ))
            }
            ExportStatus::Success(path) => {
                let display_path = {
                    let path_str = path.display().to_string();
                    if path_str.len() > 40 {
                        format!(
                            "...{}",
                            path.file_name()
                                .map(|n| format!("/{}", n.to_string_lossy()))
                                .unwrap_or_else(|| path_str)
                        )
                    } else {
                        path_str
                    }
                };
                Line::from(Span::styled(
                    format!("  ✓ Saved: {}", display_path),
                    Style::default().fg(palette.success),
                ))
            }
            ExportStatus::Error(err) => Line::from(Span::styled(
                format!("  ✗ Error: {}", err),
                Style::default().fg(palette.error),
            )),
        };
        frame.render_widget(Paragraph::new(status_line), layout[7]);
    }

    // フッター
    let footer_text = match &model.export_status {
        Some(ExportStatus::Success(_)) | Some(ExportStatus::Error(_)) => "Press Esc to close",
        _ => "Enter: Export | Tab/j/k: Switch | Esc: Cancel",
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(palette.text_dim));
    frame.render_widget(footer, layout[8]);
}
