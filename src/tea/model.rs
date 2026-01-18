use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::domain::Session;
use crate::export::ExportFormat;
use crate::search::{FilterCriteria, FilterField, SearchQuery};

/// エクスポートステータス
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportStatus {
    /// 形式選択中
    Selecting,
    /// エクスポート実行中
    Exporting,
    /// エクスポート成功
    Success(PathBuf),
    /// エクスポートエラー
    Error(String),
}

/// ビューモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// セッション一覧表示
    #[default]
    SessionList,
    /// セッション詳細表示
    SessionDetail,
    /// 検索モード
    Search,
    /// フィルタモード
    Filter,
    /// ヘルプ表示
    Help,
    /// エクスポートダイアログ
    Export,
}

/// セッションプレビュー（軽量なプレビュー情報）
#[derive(Debug, Clone)]
pub struct SessionPreview {
    /// プロジェクト名
    pub project_name: String,
    /// 開始日時（フォーマット済み）
    pub formatted_time: String,
    /// メッセージ数
    pub message_count: usize,
    /// 最初のメッセージプレビュー
    pub first_message: Option<String>,
}

impl SessionPreview {
    /// SessionListItem からプレビューを作成
    pub fn from_list_item(item: &SessionListItem) -> Self {
        Self {
            project_name: item.project_name.clone(),
            formatted_time: item.formatted_time.clone(),
            message_count: 0, // 詳細は読み込み時に更新
            first_message: Some(item.display.clone()),
        }
    }
}

/// セッション一覧の表示用アイテム
#[derive(Debug, Clone)]
pub struct SessionListItem {
    /// セッション ID
    pub session_id: String,
    /// プロジェクト名
    pub project_name: String,
    /// プロジェクトパス
    pub project_path: String,
    /// 表示テキスト（最初のユーザーメッセージ）
    pub display: String,
    /// フォーマット済み日時
    pub formatted_time: String,
    /// 日時（フィルタリング用）
    pub datetime: DateTime<Utc>,
}

/// TEA アーキテクチャの Model
/// アプリケーション全体の状態を保持する
#[derive(Debug, Clone)]
pub struct Model {
    /// セッション一覧
    pub sessions: Vec<SessionListItem>,
    /// 選択中のインデックス
    pub selected_index: usize,
    /// 終了フラグ
    pub should_quit: bool,
    /// 現在のビューモード
    pub view_mode: ViewMode,
    /// 前のビューモード（ヘルプから戻る用）
    pub previous_view_mode: ViewMode,
    /// 現在表示中のセッション
    pub current_session: Option<Session>,
    /// 詳細表示のスクロールオフセット
    pub detail_scroll_offset: usize,
    /// プレビュー用のセッション情報
    pub preview_session: Option<SessionPreview>,
    /// 検索クエリ
    pub search_query: SearchQuery,
    /// フィルタ条件
    pub filter_criteria: FilterCriteria,
    /// フィルタ適用後のインデックス一覧
    pub filtered_indices: Vec<usize>,
    /// フィルタが適用されているか
    pub is_filtered: bool,
    /// フィルタパネルの選択中フィールド
    pub filter_field: FilterField,
    /// フィルタパネルのプロジェクト名入力
    pub filter_project_input: String,
    /// 日付プリセット選択インデックス (0: All, 1: Today, 2: Last 7 days, 3: Last 30 days)
    pub date_preset_index: usize,
    /// エクスポート形式
    pub export_format: ExportFormat,
    /// エクスポートステータス
    pub export_status: Option<ExportStatus>,
    /// エラーメッセージ（セッション一覧画面で表示）
    pub error_message: Option<String>,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    /// 新規作成
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            should_quit: false,
            view_mode: ViewMode::default(),
            previous_view_mode: ViewMode::default(),
            current_session: None,
            detail_scroll_offset: 0,
            preview_session: None,
            search_query: SearchQuery::default(),
            filter_criteria: FilterCriteria::default(),
            filtered_indices: Vec::new(),
            is_filtered: false,
            filter_field: FilterField::default(),
            filter_project_input: String::new(),
            date_preset_index: 0,
            export_format: ExportFormat::default(),
            export_status: None,
            error_message: None,
        }
    }

    /// セッション一覧を設定
    pub fn with_sessions(mut self, sessions: Vec<SessionListItem>) -> Self {
        self.sessions = sessions;
        self
    }

    /// 上に移動
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 下に移動
    pub fn move_down(&mut self) {
        let max_index = if self.is_filtered {
            self.filtered_indices.len().saturating_sub(1)
        } else {
            self.sessions.len().saturating_sub(1)
        };

        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    /// 選択中のセッションを取得
    pub fn selected_session(&self) -> Option<&SessionListItem> {
        if self.is_filtered {
            let actual_index = self.filtered_indices.get(self.selected_index)?;
            self.sessions.get(*actual_index)
        } else {
            self.sessions.get(self.selected_index)
        }
    }

    /// フィルタされたセッション一覧を取得
    pub fn filtered_sessions(&self) -> Vec<&SessionListItem> {
        if self.is_filtered {
            self.filtered_indices
                .iter()
                .filter_map(|&i| self.sessions.get(i))
                .collect()
        } else {
            self.sessions.iter().collect()
        }
    }

    /// フィルタ後のセッション数を取得
    pub fn filtered_count(&self) -> usize {
        if self.is_filtered {
            self.filtered_indices.len()
        } else {
            self.sessions.len()
        }
    }

    /// 検索/フィルタをクリア
    pub fn clear_search_filter(&mut self) {
        self.search_query = SearchQuery::default();
        self.filter_criteria.clear();
        self.filtered_indices.clear();
        self.is_filtered = false;
        self.selected_index = 0;
        self.filter_project_input.clear();
        self.date_preset_index = 0;
    }

    /// 検索を適用
    pub fn apply_search(&mut self) {
        use crate::search::SearchEngine;

        self.filtered_indices = SearchEngine::search_and_filter(
            &self.sessions,
            &self.search_query,
            &self.filter_criteria,
        );
        self.is_filtered = !self.search_query.is_empty() || self.filter_criteria.is_set();
        self.selected_index = 0;
        self.update_preview();
    }

    /// フィルタを適用
    pub fn apply_filter(&mut self) {
        // プロジェクト入力をフィルタ条件に反映
        if self.filter_project_input.is_empty() {
            self.filter_criteria.project_filter = None;
        } else {
            self.filter_criteria.project_filter = Some(self.filter_project_input.clone());
        }

        self.apply_search();
    }

    /// スクロールオフセットをリセット
    pub fn reset_scroll(&mut self) {
        self.detail_scroll_offset = 0;
    }

    /// 上にスクロール
    pub fn scroll_up(&mut self, amount: usize) {
        self.detail_scroll_offset = self.detail_scroll_offset.saturating_sub(amount);
    }

    /// 下にスクロール（最大値を指定）
    pub fn scroll_down(&mut self, amount: usize, max: usize) {
        self.detail_scroll_offset = (self.detail_scroll_offset + amount).min(max);
    }

    /// 選択中のセッションからプレビューを更新
    pub fn update_preview(&mut self) {
        self.preview_session = self.selected_session().map(SessionPreview::from_list_item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sessions(count: usize) -> Vec<SessionListItem> {
        (0..count)
            .map(|i| SessionListItem {
                session_id: format!("session-{}", i),
                project_name: format!("project-{}", i),
                project_path: format!("/path/to/project-{}", i),
                display: format!("Message {}", i),
                formatted_time: "2025-01-01 00:00".to_string(),
                datetime: Utc::now(),
            })
            .collect()
    }

    #[test]
    fn test_model_new() {
        let model = Model::new();
        assert!(model.sessions.is_empty());
        assert_eq!(model.selected_index, 0);
        assert!(!model.should_quit);
    }

    #[test]
    fn test_model_with_sessions() {
        let sessions = create_test_sessions(3);
        let model = Model::new().with_sessions(sessions);
        assert_eq!(model.sessions.len(), 3);
    }

    #[test]
    fn test_move_up() {
        let sessions = create_test_sessions(5);
        let mut model = Model::new().with_sessions(sessions);
        model.selected_index = 2;

        model.move_up();
        assert_eq!(model.selected_index, 1);

        model.move_up();
        assert_eq!(model.selected_index, 0);

        // 先頭では動かない
        model.move_up();
        assert_eq!(model.selected_index, 0);
    }

    #[test]
    fn test_move_down() {
        let sessions = create_test_sessions(3);
        let mut model = Model::new().with_sessions(sessions);

        model.move_down();
        assert_eq!(model.selected_index, 1);

        model.move_down();
        assert_eq!(model.selected_index, 2);

        // 末尾では動かない
        model.move_down();
        assert_eq!(model.selected_index, 2);
    }

    #[test]
    fn test_move_down_empty() {
        let mut model = Model::new();
        model.move_down();
        assert_eq!(model.selected_index, 0);
    }

    #[test]
    fn test_selected_session() {
        let sessions = create_test_sessions(3);
        let mut model = Model::new().with_sessions(sessions);

        let selected = model.selected_session().unwrap();
        assert_eq!(selected.session_id, "session-0");

        model.selected_index = 2;
        let selected = model.selected_session().unwrap();
        assert_eq!(selected.session_id, "session-2");
    }

    #[test]
    fn test_selected_session_empty() {
        let model = Model::new();
        assert!(model.selected_session().is_none());
    }

    #[test]
    fn test_scroll_up() {
        let mut model = Model::new();
        model.detail_scroll_offset = 5;

        model.scroll_up(3);
        assert_eq!(model.detail_scroll_offset, 2);

        model.scroll_up(5);
        assert_eq!(model.detail_scroll_offset, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut model = Model::new();

        model.scroll_down(3, 100);
        assert_eq!(model.detail_scroll_offset, 3);

        model.scroll_down(100, 50);
        assert_eq!(model.detail_scroll_offset, 50);
    }

    #[test]
    fn test_reset_scroll() {
        let mut model = Model::new();
        model.detail_scroll_offset = 10;

        model.reset_scroll();
        assert_eq!(model.detail_scroll_offset, 0);
    }
}
