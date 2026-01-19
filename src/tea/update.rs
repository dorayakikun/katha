use super::message::Message;
use super::model::{ExportStatus, Model, TreeNodeKind, ViewMode};
use crate::search::DateRange;

/// TEA の update 関数
/// Message を受け取り Model を更新する純粋関数
pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::Initialized => {
            // 初期化完了時にプレビューを更新
            model.update_preview();
        }
        Message::SelectSession(index) => {
            if index < model.sessions.len() {
                model.selected_index = index;
            }
        }
        Message::MoveUp => {
            model.move_up();
            model.update_preview();
        }
        Message::MoveDown => {
            model.move_down();
            model.update_preview();
        }
        Message::EnterDetail => {
            // ツリーアイテムが選択されている場合
            if let Some(item) = model.selected_tree_item() {
                match item.kind {
                    TreeNodeKind::Project => {
                        // プロジェクトノードの場合は展開/折りたたみ
                        let project_path = item.project_path.clone();
                        model.toggle_project(&project_path);
                        model.update_preview();
                    }
                    TreeNodeKind::Session => {
                        // セッションノードの場合は詳細画面に遷移
                        model.view_mode = ViewMode::SessionDetail;
                        model.reset_detail_cursor();
                    }
                }
            } else if model.selected_session().is_some() {
                // 旧方式（互換性のため）
                model.view_mode = ViewMode::SessionDetail;
                model.reset_detail_cursor();
            }
        }
        Message::BackToList => {
            // 一覧画面に戻る
            model.view_mode = ViewMode::SessionList;
            model.current_session = None;
            model.reset_detail_cursor();
        }
        Message::ScrollUp(amount) => {
            model.move_detail_cursor_up(amount);
        }
        Message::ScrollDown(amount) => {
            model.move_detail_cursor_down(amount);
        }
        Message::CopySelectedMessage => {}
        Message::CopySelectedMessageWithMeta => {}
        Message::SessionLoaded(session) => {
            model.current_session = Some(session);
            model.reset_detail_cursor();
        }
        Message::SessionLoadFailed(_error) => {
            // エラー時は一覧に戻る
            model.view_mode = ViewMode::SessionList;
            model.current_session = None;
        }
        Message::Quit => {
            model.should_quit = true;
        }
        Message::None => {
            // 何もしない
        }

        // === 検索関連 ===
        Message::StartSearch => {
            model.view_mode = ViewMode::Search;
        }
        Message::CancelSearch => {
            model.view_mode = ViewMode::SessionList;
            model.search_query.text.clear();
            model.apply_search(); // 検索をクリアして再適用
        }
        Message::SearchInput(c) => {
            model.search_query.text.push(c);
            model.apply_search(); // インクリメンタル検索
        }
        Message::SearchBackspace => {
            model.search_query.text.pop();
            model.apply_search(); // インクリメンタル検索
        }
        Message::ConfirmSearch => {
            model.view_mode = ViewMode::SessionList;
            model.apply_search();
        }

        // === フィルタ関連 ===
        Message::StartFilter => {
            model.view_mode = ViewMode::Filter;
            // フィルタパネル表示時に現在の状態を入力フィールドに反映
            model.filter_project_input = model
                .filter_criteria
                .project_filter
                .clone()
                .unwrap_or_default();
        }
        Message::CancelFilter => {
            model.view_mode = ViewMode::SessionList;
        }
        Message::ApplyFilter => {
            model.view_mode = ViewMode::SessionList;
            model.apply_filter();
        }
        Message::ClearFilter => {
            if model.is_filtered || !model.search_query.is_empty() {
                model.clear_search_filter();
            } else {
                // フィルタがなければ終了
                model.should_quit = true;
            }
        }
        Message::FilterNextField => {
            model.filter_field = model.filter_field.next();
        }
        Message::FilterDatePresetNext => {
            if model.date_preset_index < 3 {
                model.date_preset_index += 1;
                update_date_range_from_preset(model);
            }
        }
        Message::FilterDatePresetPrev => {
            if model.date_preset_index > 0 {
                model.date_preset_index -= 1;
                update_date_range_from_preset(model);
            }
        }
        Message::FilterProjectInput(c) => {
            model.filter_project_input.push(c);
        }
        Message::FilterProjectBackspace => {
            model.filter_project_input.pop();
        }

        // === ヘルプ関連 ===
        Message::ShowHelp => {
            model.previous_view_mode = model.view_mode;
            model.view_mode = ViewMode::Help;
        }
        Message::CloseHelp => {
            model.view_mode = model.previous_view_mode;
        }

        // === エクスポート関連 ===
        Message::StartExport => {
            model.previous_view_mode = model.view_mode;
            model.view_mode = ViewMode::Export;
            model.export_status = Some(ExportStatus::Selecting);
        }
        Message::SelectExportFormat(format) => {
            model.export_format = format;
        }
        Message::ToggleExportFormat => {
            model.export_format = model.export_format.next();
        }
        Message::ConfirmExport => {
            model.export_status = Some(ExportStatus::Exporting);
        }
        Message::CancelExport => {
            model.view_mode = model.previous_view_mode;
            model.export_status = None;
        }
        Message::ExportCompleted(path) => {
            model.export_status = Some(ExportStatus::Success(path));
        }
        Message::ExportFailed(error) => {
            model.export_status = Some(ExportStatus::Error(error));
        }

        // === エラー関連 ===
        Message::ShowError(error) => {
            model.error_message = Some(error);
        }
        Message::ClearError => {
            model.error_message = None;
        }

        // === ツリー操作関連 ===
        Message::ToggleProject(project_path) => {
            model.toggle_project(&project_path);
            model.update_preview();
        }
        Message::ExpandCurrentProject => {
            model.expand_current_project();
            model.update_preview();
        }
        Message::CollapseCurrentProject => {
            model.collapse_current_project();
            model.update_preview();
        }
        Message::ExpandAll => {
            model.expand_all();
            model.update_preview();
        }
        Message::CollapseAll => {
            model.collapse_all();
            model.update_preview();
        }
    }
}

/// 日付プリセットインデックスから DateRange を更新
fn update_date_range_from_preset(model: &mut Model) {
    model.filter_criteria.date_range = match model.date_preset_index {
        0 => DateRange::default(), // All
        1 => DateRange::today(),
        2 => DateRange::last_week(),
        3 => DateRange::last_month(),
        _ => DateRange::default(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::tea::SessionListItem;

    fn create_test_model() -> Model {
        let sessions = (0..5)
            .map(|i| SessionListItem {
                session_id: format!("session-{}", i),
                project_name: format!("project-{}", i),
                project_path: format!("/path/to/project-{}", i),
                latest_user_message: format!("Message {}", i),
                formatted_time: "2025-01-01 00:00".to_string(),
                datetime: Utc::now(),
            })
            .collect();

        Model::new().with_sessions(sessions)
    }

    #[test]
    fn test_update_initialized() {
        let mut model = create_test_model();
        update(&mut model, Message::Initialized);
        // 状態は変わらない
        assert_eq!(model.selected_index, 0);
        assert!(!model.should_quit);
    }

    #[test]
    fn test_update_select_session() {
        let mut model = create_test_model();

        update(&mut model, Message::SelectSession(3));
        assert_eq!(model.selected_index, 3);

        // 範囲外は無視
        update(&mut model, Message::SelectSession(100));
        assert_eq!(model.selected_index, 3);
    }

    #[test]
    fn test_update_move_up() {
        let mut model = create_test_model();
        model.selected_index = 2;

        update(&mut model, Message::MoveUp);
        assert_eq!(model.selected_index, 1);
    }

    #[test]
    fn test_update_move_down() {
        let mut model = create_test_model();

        update(&mut model, Message::MoveDown);
        assert_eq!(model.selected_index, 1);
    }

    #[test]
    fn test_update_quit() {
        let mut model = create_test_model();
        assert!(!model.should_quit);

        update(&mut model, Message::Quit);
        assert!(model.should_quit);
    }

    #[test]
    fn test_update_none() {
        let mut model = create_test_model();
        model.selected_index = 2;

        update(&mut model, Message::None);
        // 状態は変わらない
        assert_eq!(model.selected_index, 2);
        assert!(!model.should_quit);
    }
}
