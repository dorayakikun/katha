use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::domain::Session;
use crate::export::ExportFormat;
use crate::search::{FilterCriteria, FilterField, SearchQuery};

/// ツリーノードの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeKind {
    /// プロジェクトノード
    Project,
    /// セッションノード
    Session,
}

/// ツリー表示用のアイテム
#[derive(Debug, Clone)]
pub struct TreeItem {
    /// ノードの種類
    pub kind: TreeNodeKind,
    /// プロジェクトパス
    pub project_path: String,
    /// プロジェクト名
    pub project_name: String,
    /// セッション情報（Session ノードの場合のみ）
    pub session: Option<SessionListItem>,
    /// 子ノード数（Project ノードの場合のセッション数）
    pub child_count: usize,
    /// 最新の日時
    pub latest_datetime: DateTime<Utc>,
    /// フォーマット済み日時
    pub formatted_time: String,
}

impl TreeItem {
    /// プロジェクトノードを作成
    pub fn project(group: &ProjectGroup) -> Self {
        let latest = group
            .sessions
            .first()
            .map(|s| (s.datetime, s.formatted_time.clone()))
            .unwrap_or_else(|| (Utc::now(), String::new()));

        Self {
            kind: TreeNodeKind::Project,
            project_path: group.project_path.clone(),
            project_name: group.project_name.clone(),
            session: None,
            child_count: group.sessions.len(),
            latest_datetime: latest.0,
            formatted_time: latest.1,
        }
    }

    /// セッションノードを作成
    pub fn session(item: &SessionListItem) -> Self {
        Self {
            kind: TreeNodeKind::Session,
            project_path: item.project_path.clone(),
            project_name: item.project_name.clone(),
            session: Some(item.clone()),
            child_count: 0,
            latest_datetime: item.datetime,
            formatted_time: item.formatted_time.clone(),
        }
    }
}

/// プロジェクトグループ
#[derive(Debug, Clone)]
pub struct ProjectGroup {
    /// プロジェクトパス
    pub project_path: String,
    /// プロジェクト名
    pub project_name: String,
    /// セッション一覧（新しい順にソート済み）
    pub sessions: Vec<SessionListItem>,
}

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
    /// 最新のユーザーメッセージプレビュー
    pub latest_user_message: Option<String>,
}

impl SessionPreview {
    /// SessionListItem からプレビューを作成
    pub fn from_list_item(item: &SessionListItem) -> Self {
        Self {
            project_name: item.project_name.clone(),
            formatted_time: item.formatted_time.clone(),
            message_count: 0, // 詳細は読み込み時に更新
            latest_user_message: Some(item.latest_user_message.clone()),
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
    /// 表示テキスト（最新のユーザーメッセージ）
    pub latest_user_message: String,
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
    /// プロジェクトグループ一覧
    pub project_groups: Vec<ProjectGroup>,
    /// フィルタ適用後のプロジェクトグループ一覧
    pub filtered_project_groups: Vec<ProjectGroup>,
    /// 展開されているプロジェクトのパス
    pub expanded_projects: HashSet<String>,
    /// フィルタ適用前の展開状態
    pub expanded_projects_before_filter: Option<HashSet<String>>,
    /// ツリー表示用アイテム一覧
    pub tree_items: Vec<TreeItem>,
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
            project_groups: Vec::new(),
            filtered_project_groups: Vec::new(),
            expanded_projects: HashSet::new(),
            expanded_projects_before_filter: None,
            tree_items: Vec::new(),
        }
    }

    /// セッション一覧を設定
    pub fn with_sessions(mut self, sessions: Vec<SessionListItem>) -> Self {
        self.sessions = sessions;
        self
    }

    /// プロジェクトグループを設定
    pub fn with_project_groups(mut self, groups: Vec<ProjectGroup>) -> Self {
        self.project_groups = groups;
        self.sessions = self
            .project_groups
            .iter()
            .flat_map(|group| group.sessions.iter().cloned())
            .collect();
        self.filtered_project_groups.clear();
        self.rebuild_tree_items();
        self
    }

    /// 展開状態に基づいてツリーアイテムを再構築
    pub fn rebuild_tree_items(&mut self) {
        self.tree_items.clear();

        let groups = self.active_project_groups().clone();
        for group in &groups {
            // プロジェクトノードを追加
            self.tree_items.push(TreeItem::project(group));

            // 展開されている場合はセッションノードを追加
            if self.expanded_projects.contains(&group.project_path) {
                for session in &group.sessions {
                    self.tree_items.push(TreeItem::session(session));
                }
            }
        }

        // selected_index が範囲外にならないようにする
        if !self.tree_items.is_empty() && self.selected_index >= self.tree_items.len() {
            self.selected_index = self.tree_items.len() - 1;
        }
    }

    fn active_project_groups(&self) -> &Vec<ProjectGroup> {
        if self.is_filtered {
            &self.filtered_project_groups
        } else {
            &self.project_groups
        }
    }

    /// 選択中のツリーアイテムを取得
    pub fn selected_tree_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.selected_index)
    }

    /// 総セッション数を取得
    pub fn total_session_count(&self) -> usize {
        self.project_groups
            .iter()
            .map(|g| g.sessions.len())
            .sum()
    }

    /// プロジェクトの展開/折りたたみを切り替え
    pub fn toggle_project(&mut self, project_path: &str) {
        if self.expanded_projects.contains(project_path) {
            self.expanded_projects.remove(project_path);
        } else {
            self.expanded_projects.insert(project_path.to_string());
        }
        self.rebuild_tree_items();
    }

    /// 選択中のプロジェクトを展開
    pub fn expand_current_project(&mut self) {
        if let Some(item) = self.selected_tree_item() {
            if item.kind == TreeNodeKind::Project
                && !self.expanded_projects.contains(&item.project_path)
            {
                let path = item.project_path.clone();
                self.expanded_projects.insert(path);
                self.rebuild_tree_items();
            }
        }
    }

    /// 選択中のプロジェクトを折りたたみ
    pub fn collapse_current_project(&mut self) {
        if let Some(item) = self.selected_tree_item() {
            let project_path = item.project_path.clone();
            if self.expanded_projects.contains(&project_path) {
                // セッションノードの場合は親プロジェクトに移動してから折りたたみ
                if item.kind == TreeNodeKind::Session {
                    // 親プロジェクトを探す
                    if let Some(project_index) = self
                        .tree_items
                        .iter()
                        .position(|t| t.kind == TreeNodeKind::Project && t.project_path == project_path)
                    {
                        self.selected_index = project_index;
                    }
                }
                self.expanded_projects.remove(&project_path);
                self.rebuild_tree_items();
            }
        }
    }

    /// すべてのプロジェクトを展開
    pub fn expand_all(&mut self) {
        for group in &self.project_groups {
            self.expanded_projects.insert(group.project_path.clone());
        }
        self.rebuild_tree_items();
    }

    /// すべてのプロジェクトを折りたたみ
    pub fn collapse_all(&mut self) {
        self.expanded_projects.clear();
        // 選択位置を調整（プロジェクトのみになるので）
        if let Some(item) = self.selected_tree_item() {
            // 現在選択中のプロジェクトを探す
            let project_path = item.project_path.clone();
            self.rebuild_tree_items();
            // 同じプロジェクトを選択
            if let Some(idx) = self.tree_items.iter().position(|t| t.project_path == project_path) {
                self.selected_index = idx;
            } else {
                self.selected_index = 0;
            }
        } else {
            self.rebuild_tree_items();
            self.selected_index = 0;
        }
    }

    /// 上に移動
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 下に移動
    pub fn move_down(&mut self) {
        let max_index = if !self.tree_items.is_empty() {
            self.tree_items.len().saturating_sub(1)
        } else if self.is_filtered {
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
        if let Some(item) = self.selected_tree_item() {
            if item.kind == TreeNodeKind::Session {
                return item.session.as_ref();
            }
        }

        if self.is_filtered {
            let actual_index = self.filtered_indices.get(self.selected_index)?;
            return self.sessions.get(*actual_index);
        }

        self.sessions.get(self.selected_index)
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
        self.filtered_project_groups.clear();
        self.restore_expanded_projects_after_filter();
        self.rebuild_tree_items();
        self.update_preview();
    }

    /// 検索を適用
    pub fn apply_search(&mut self) {
        use crate::search::SearchEngine;

        self.filtered_indices = SearchEngine::search_and_filter(
            &self.sessions,
            &self.search_query,
            &self.filter_criteria,
        );
        let was_filtered = self.is_filtered;
        self.is_filtered = !self.search_query.is_empty() || self.filter_criteria.is_set();
        self.rebuild_filtered_project_groups();
        self.sync_expanded_projects_for_filter(was_filtered);
        self.rebuild_tree_items();
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

    fn rebuild_filtered_project_groups(&mut self) {
        if !self.is_filtered {
            self.filtered_project_groups.clear();
            return;
        }

        let mut grouped: HashMap<String, Vec<SessionListItem>> = HashMap::new();
        for index in &self.filtered_indices {
            if let Some(session) = self.sessions.get(*index) {
                grouped
                    .entry(session.project_path.clone())
                    .or_default()
                    .push(session.clone());
            }
        }

        let mut filtered_groups = Vec::new();
        for group in &self.project_groups {
            if let Some(sessions) = grouped.remove(&group.project_path) {
                filtered_groups.push(ProjectGroup {
                    project_path: group.project_path.clone(),
                    project_name: group.project_name.clone(),
                    sessions,
                });
            }
        }

        self.filtered_project_groups = filtered_groups;
    }

    fn sync_expanded_projects_for_filter(&mut self, was_filtered: bool) {
        if self.is_filtered {
            if !was_filtered {
                self.expanded_projects_before_filter = Some(self.expanded_projects.clone());
            }
            self.expanded_projects = self
                .filtered_project_groups
                .iter()
                .map(|group| group.project_path.clone())
                .collect();
        } else if was_filtered {
            self.restore_expanded_projects_after_filter();
        }
    }

    fn restore_expanded_projects_after_filter(&mut self) {
        if let Some(previous) = self.expanded_projects_before_filter.take() {
            self.expanded_projects = previous;
        }
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
        // ツリーアイテムから選択中のセッションを取得
        if let Some(item) = self.selected_tree_item() {
            match item.kind {
                TreeNodeKind::Project => {
                    // プロジェクトノードの場合は最新セッションのプレビューを表示
                    if let Some(group) = self
                        .active_project_groups()
                        .iter()
                        .find(|g| g.project_path == item.project_path)
                    {
                        self.preview_session =
                            group.sessions.first().map(SessionPreview::from_list_item);
                    } else {
                        self.preview_session = None;
                    }
                }
                TreeNodeKind::Session => {
                    // セッションノードの場合はそのセッションのプレビューを表示
                    self.preview_session =
                        item.session.as_ref().map(SessionPreview::from_list_item);
                }
            }
        } else {
            // 旧方式（互換性のため）
            self.preview_session = self.selected_session().map(SessionPreview::from_list_item);
        }
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
                latest_user_message: format!("Message {}", i),
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

    // TreeNodeKind のテスト
    #[test]
    fn test_tree_node_kind_equality() {
        assert_eq!(TreeNodeKind::Project, TreeNodeKind::Project);
        assert_eq!(TreeNodeKind::Session, TreeNodeKind::Session);
        assert_ne!(TreeNodeKind::Project, TreeNodeKind::Session);
    }

    // ProjectGroup と TreeItem のテスト
    fn create_project_group(name: &str, session_count: usize) -> ProjectGroup {
        let sessions: Vec<SessionListItem> = (0..session_count)
            .map(|i| SessionListItem {
                session_id: format!("{}-session-{}", name, i),
                project_name: name.to_string(),
                project_path: format!("/path/to/{}", name),
                latest_user_message: format!("Message {} for {}", i, name),
                formatted_time: format!("2025-01-0{} 00:00", i + 1),
                datetime: Utc::now(),
            })
            .collect();

        ProjectGroup {
            project_path: format!("/path/to/{}", name),
            project_name: name.to_string(),
            sessions,
        }
    }

    #[test]
    fn test_project_group_creation() {
        let group = create_project_group("test-project", 3);
        assert_eq!(group.project_name, "test-project");
        assert_eq!(group.project_path, "/path/to/test-project");
        assert_eq!(group.sessions.len(), 3);
    }

    #[test]
    fn test_tree_item_project() {
        let group = create_project_group("my-project", 5);
        let tree_item = TreeItem::project(&group);

        assert_eq!(tree_item.kind, TreeNodeKind::Project);
        assert_eq!(tree_item.project_name, "my-project");
        assert_eq!(tree_item.project_path, "/path/to/my-project");
        assert!(tree_item.session.is_none());
        assert_eq!(tree_item.child_count, 5);
    }

    #[test]
    fn test_tree_item_session() {
        let session = SessionListItem {
            session_id: "test-session-id".to_string(),
            project_name: "test-project".to_string(),
            project_path: "/path/to/test-project".to_string(),
            latest_user_message: "Hello, world!".to_string(),
            formatted_time: "2025-01-15 10:30".to_string(),
            datetime: Utc::now(),
        };

        let tree_item = TreeItem::session(&session);

        assert_eq!(tree_item.kind, TreeNodeKind::Session);
        assert_eq!(tree_item.project_name, "test-project");
        assert_eq!(tree_item.project_path, "/path/to/test-project");
        assert!(tree_item.session.is_some());
        assert_eq!(tree_item.session.unwrap().session_id, "test-session-id");
        assert_eq!(tree_item.child_count, 0);
    }

    #[test]
    fn test_tree_item_project_empty_sessions() {
        let group = ProjectGroup {
            project_path: "/path/to/empty".to_string(),
            project_name: "empty-project".to_string(),
            sessions: vec![],
        };

        let tree_item = TreeItem::project(&group);

        assert_eq!(tree_item.kind, TreeNodeKind::Project);
        assert_eq!(tree_item.child_count, 0);
    }

    // Model 階層表示関連のテスト
    #[test]
    fn test_model_with_project_groups() {
        let groups = vec![
            create_project_group("project-a", 2),
            create_project_group("project-b", 3),
        ];
        let model = Model::new().with_project_groups(groups);

        assert_eq!(model.project_groups.len(), 2);
        // 初期状態は折りたたみなので、プロジェクトノードのみ
        assert_eq!(model.tree_items.len(), 2);
        assert_eq!(model.tree_items[0].kind, TreeNodeKind::Project);
        assert_eq!(model.tree_items[1].kind, TreeNodeKind::Project);
    }

    #[test]
    fn test_model_toggle_project() {
        let groups = vec![
            create_project_group("project-a", 2),
            create_project_group("project-b", 3),
        ];
        let mut model = Model::new().with_project_groups(groups);

        // 展開前: 2 プロジェクト
        assert_eq!(model.tree_items.len(), 2);

        // project-a を展開
        model.toggle_project("/path/to/project-a");
        // 2 プロジェクト + 2 セッション = 4
        assert_eq!(model.tree_items.len(), 4);
        assert_eq!(model.tree_items[0].kind, TreeNodeKind::Project);
        assert_eq!(model.tree_items[1].kind, TreeNodeKind::Session);
        assert_eq!(model.tree_items[2].kind, TreeNodeKind::Session);
        assert_eq!(model.tree_items[3].kind, TreeNodeKind::Project);

        // project-a を折りたたみ
        model.toggle_project("/path/to/project-a");
        assert_eq!(model.tree_items.len(), 2);
    }

    #[test]
    fn test_model_expand_all() {
        let groups = vec![
            create_project_group("project-a", 2),
            create_project_group("project-b", 3),
        ];
        let mut model = Model::new().with_project_groups(groups);

        model.expand_all();
        // 2 プロジェクト + 2 セッション + 3 セッション = 7
        assert_eq!(model.tree_items.len(), 7);
    }

    #[test]
    fn test_model_collapse_all() {
        let groups = vec![
            create_project_group("project-a", 2),
            create_project_group("project-b", 3),
        ];
        let mut model = Model::new().with_project_groups(groups);

        model.expand_all();
        assert_eq!(model.tree_items.len(), 7);

        model.collapse_all();
        assert_eq!(model.tree_items.len(), 2);
    }

    #[test]
    fn test_model_total_session_count() {
        let groups = vec![
            create_project_group("project-a", 2),
            create_project_group("project-b", 3),
        ];
        let model = Model::new().with_project_groups(groups);

        assert_eq!(model.total_session_count(), 5);
    }

    #[test]
    fn test_model_selected_tree_item() {
        let groups = vec![create_project_group("project-a", 2)];
        let mut model = Model::new().with_project_groups(groups);

        // 最初はプロジェクトが選択されている
        let item = model.selected_tree_item().unwrap();
        assert_eq!(item.kind, TreeNodeKind::Project);

        // 展開して移動
        model.toggle_project("/path/to/project-a");
        model.selected_index = 1;
        let item = model.selected_tree_item().unwrap();
        assert_eq!(item.kind, TreeNodeKind::Session);
    }

    #[test]
    fn test_model_move_down_with_tree_items() {
        let groups = vec![create_project_group("project-a", 2)];
        let mut model = Model::new().with_project_groups(groups);
        model.toggle_project("/path/to/project-a");

        // tree_items: [Project, Session, Session]
        assert_eq!(model.selected_index, 0);

        model.move_down();
        assert_eq!(model.selected_index, 1);

        model.move_down();
        assert_eq!(model.selected_index, 2);

        // 末尾では動かない
        model.move_down();
        assert_eq!(model.selected_index, 2);
    }

    #[test]
    fn test_model_expand_current_project() {
        let groups = vec![create_project_group("project-a", 2)];
        let mut model = Model::new().with_project_groups(groups);

        assert_eq!(model.tree_items.len(), 1);

        model.expand_current_project();
        assert_eq!(model.tree_items.len(), 3);
    }

    #[test]
    fn test_model_collapse_current_project() {
        let groups = vec![create_project_group("project-a", 2)];
        let mut model = Model::new().with_project_groups(groups);
        model.toggle_project("/path/to/project-a");

        // セッションを選択
        model.selected_index = 1;
        model.collapse_current_project();

        // 折りたたまれてプロジェクトが選択される
        assert_eq!(model.tree_items.len(), 1);
        assert_eq!(model.selected_index, 0);
    }
}
