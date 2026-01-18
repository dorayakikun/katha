use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::layout::TwoPane;
use crate::tea::{Model, ViewMode};
use crate::views::render_preview_pane;
use crate::widgets::{ProjectTree, ProjectTreeState, SearchBar, StatusBar};

/// セッション一覧ビューをレンダリング
pub fn render_session_list(frame: &mut Frame, model: &Model) {
    let area = frame.area();

    // 検索モード時は検索バー用の行を追加
    let is_search_mode = model.view_mode == ViewMode::Search;

    let layout = if is_search_mode {
        // 4分割レイアウト: ヘッダー(3行) | 検索バー(1行) | 一覧(可変) | フッター(3行)
        Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area)
    } else {
        // 3分割レイアウト: ヘッダー(3行) | 一覧(可変) | フッター(3行)
        Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area)
    };

    render_header(frame, layout[0], model);

    if is_search_mode {
        render_search_bar(frame, layout[1], model);
        render_main_content(frame, layout[2], model);
        render_footer(frame, layout[3], model);
    } else {
        render_main_content(frame, layout[1], model);
        render_footer(frame, layout[2], model);
    }

    // フィルタモード時はオーバーレイでフィルタパネルを表示
    if model.view_mode == ViewMode::Filter {
        render_filter_panel(frame, model);
    }

    // エラーメッセージがある場合は画面下部に表示
    if let Some(error) = &model.error_message {
        render_error_message(frame, error);
    }
}

/// メインコンテンツ領域をレンダリング（2ペイン）
fn render_main_content(frame: &mut Frame, area: Rect, model: &Model) {
    let two_pane = TwoPane::new(55, 45);
    let (left, right) = two_pane.split(area);

    render_sessions(frame, left, model);
    render_preview_pane(frame, right, model);
}

/// ヘッダーをレンダリング
fn render_header(frame: &mut Frame, area: Rect, model: &Model) {
    let mut spans = vec![
        Span::styled(
            " katha ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "- Claude Code Conversation Viewer",
            Style::default().fg(Color::Gray),
        ),
    ];

    // フィルタ/検索が適用されている場合は件数を表示
    if model.is_filtered || !model.search_query.is_empty() {
        let count_text = format!("  ({}/{})", model.filtered_count(), model.sessions.len());
        spans.push(Span::styled(count_text, Style::default().fg(Color::Yellow)));
    }

    let title = Line::from(spans);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let header = Paragraph::new(title).block(block);
    frame.render_widget(header, area);
}

/// 検索バーをレンダリング
fn render_search_bar(frame: &mut Frame, area: Rect, model: &Model) {
    let search_bar = SearchBar::new(&model.search_query.text).cursor_visible(true);
    frame.render_widget(search_bar, area);
}

/// セッション一覧をレンダリング
fn render_sessions(frame: &mut Frame, area: Rect, model: &Model) {
    let project_count = model.project_groups.len();
    let session_count = model.total_session_count();

    let title = if model.is_filtered || !model.search_query.is_empty() {
        let filtered_sessions = model.filtered_sessions();
        format!(
            " Sessions ({}/{}) ",
            filtered_sessions.len(),
            session_count
        )
    } else {
        format!(" Projects ({}) / Sessions ({}) ", project_count, session_count)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if model.tree_items.is_empty() {
        let message = if model.project_groups.is_empty() {
            "No sessions found"
        } else {
            "No matching sessions"
        };
        let empty_message = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty_message, area);
        return;
    }

    // ProjectTree ウィジェットを使用
    let tree = ProjectTree::new(&model.tree_items, &model.expanded_projects).block(block);

    let mut state = ProjectTreeState::new();
    state.select(model.selected_index);

    frame.render_stateful_widget(tree, area, &mut state);
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

        let keys: Vec<(&str, &str)> = match model.view_mode {
            ViewMode::Search => vec![("Enter", "Confirm"), ("Esc", "Cancel")],
            ViewMode::Filter => vec![
                ("Tab", "Switch"),
                ("Enter", "Apply"),
                ("Esc", "Cancel"),
                ("c", "Clear"),
            ],
            _ => {
                let mut keys = vec![
                    ("j/k", "Move"),
                    ("Enter", "Open/Toggle"),
                    ("l/h", "Expand/Fold"),
                    ("/", "Search"),
                    ("f", "Filter"),
                    ("e", "Export"),
                    ("?", "Help"),
                ];
                if model.is_filtered || !model.search_query.is_empty() {
                    keys.push(("Esc", "Clear"));
                }
                keys.push(("q", "Quit"));
                keys
            }
        };

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

/// フィルタパネルをレンダリング（オーバーレイ）
fn render_filter_panel(frame: &mut Frame, model: &Model) {
    use crate::search::FilterField;

    let area = frame.area();

    // 中央にパネルを配置
    let panel_width = 50.min(area.width.saturating_sub(4));
    let panel_height = 12.min(area.height.saturating_sub(4));

    let panel_x = (area.width.saturating_sub(panel_width)) / 2;
    let panel_y = (area.height.saturating_sub(panel_height)) / 2;

    let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);

    // 背景をクリア
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, panel_area);

    // パネルブロック
    let block = Block::default()
        .title(" Filter ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(panel_area);
    frame.render_widget(block, panel_area);

    // 内部コンテンツ
    let lines_layout = Layout::vertical([
        Constraint::Length(1), // Date Range label
        Constraint::Length(4), // Date Range options
        Constraint::Length(1), // Project label
        Constraint::Length(1), // Project input
        Constraint::Min(1),    // Help
    ])
    .split(inner);

    // Date Range ラベル
    let date_style = if model.filter_field == FilterField::DateRange {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let date_label = Paragraph::new("Date Range:").style(date_style);
    frame.render_widget(date_label, lines_layout[0]);

    // Date Range オプション
    let date_presets = ["All", "Today", "Last 7 days", "Last 30 days"];
    let date_lines: Vec<Line> = date_presets
        .iter()
        .enumerate()
        .map(|(i, preset)| {
            let marker = if i == model.date_preset_index {
                "● "
            } else {
                "○ "
            };
            let style =
                if i == model.date_preset_index && model.filter_field == FilterField::DateRange {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                };
            Line::from(Span::styled(format!("  {}{}", marker, preset), style))
        })
        .collect();
    let date_options = Paragraph::new(date_lines);
    frame.render_widget(date_options, lines_layout[1]);

    // Project ラベル
    let project_style = if model.filter_field == FilterField::Project {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let project_label = Paragraph::new("Project:").style(project_style);
    frame.render_widget(project_label, lines_layout[2]);

    // Project 入力
    let cursor = if model.filter_field == FilterField::Project {
        "_"
    } else {
        ""
    };
    let project_input = Paragraph::new(format!("  {}{}", model.filter_project_input, cursor))
        .style(Style::default().fg(Color::White));
    frame.render_widget(project_input, lines_layout[3]);

    // ヘルプ
    let help = Paragraph::new("Tab: Switch | j/k: Select | Enter: Apply | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, lines_layout[4]);
}

/// エラーメッセージをレンダリング（画面下部にオーバーレイ）
fn render_error_message(frame: &mut Frame, error: &str) {
    let area = frame.area();

    // 画面下部にエラーメッセージを表示（フッターの上）
    let error_height = 3;
    let error_y = area.height.saturating_sub(error_height + 3); // フッター（3行）の上

    let error_area = Rect::new(area.x + 1, error_y, area.width.saturating_sub(2), error_height);

    // 背景をクリア
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, error_area);

    // エラーメッセージブロック
    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let error_text = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .block(block);

    frame.render_widget(error_text, error_area);
}
