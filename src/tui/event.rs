use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::KathaError;
use crate::tea::{ExportStatus, Message, ViewMode};

/// イベントハンドラ
/// キーイベントを TEA の Message に変換
pub struct EventHandler {
    /// ポーリングタイムアウト
    timeout: Duration,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// 新規作成
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_millis(100),
        }
    }

    /// イベントをポーリングして Message に変換
    pub fn poll(
        &self,
        view_mode: ViewMode,
        export_status: Option<&ExportStatus>,
    ) -> Result<Message, KathaError> {
        // Export中は短いタイムアウトでスピナーアニメーションを更新
        let timeout = if view_mode == ViewMode::Export {
            Duration::from_millis(16) // ~60fps for smooth spinner
        } else {
            self.timeout
        };

        if event::poll(timeout).map_err(|e| KathaError::Terminal(e.to_string()))?
            && let Event::Key(key) =
                event::read().map_err(|e| KathaError::Terminal(e.to_string()))?
        {
            return Ok(self.key_to_message(key, view_mode, export_status));
        }
        Ok(Message::None)
    }

    /// キーイベントを Message に変換
    fn key_to_message(
        &self,
        key: KeyEvent,
        view_mode: ViewMode,
        export_status: Option<&ExportStatus>,
    ) -> Message {
        // Ctrl+C は常に終了
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Message::Quit;
        }

        match view_mode {
            ViewMode::SessionList => self.session_list_key(key),
            ViewMode::SessionDetail => self.session_detail_key(key),
            ViewMode::Search => self.search_mode_key(key),
            ViewMode::Filter => self.filter_mode_key(key),
            ViewMode::Help => self.help_mode_key(key),
            ViewMode::Export => self.export_mode_key(key, export_status),
        }
    }

    /// セッション一覧画面のキーマッピング
    fn session_list_key(&self, key: KeyEvent) -> Message {
        match key.code {
            // 終了
            KeyCode::Char('q') => Message::Quit,
            // Esc でフィルタクリア、またはクリア済みなら終了
            KeyCode::Esc => Message::ClearFilter,
            // 上に移動
            KeyCode::Char('k') | KeyCode::Up => Message::MoveUp,
            // 下に移動
            KeyCode::Char('j') | KeyCode::Down => Message::MoveDown,
            // Enter で詳細画面へ
            KeyCode::Enter => Message::EnterDetail,
            // 検索モード開始
            KeyCode::Char('/') => Message::StartSearch,
            // フィルタモード開始
            KeyCode::Char('f') => Message::StartFilter,
            // エクスポートダイアログ表示
            KeyCode::Char('e') => Message::StartExport,
            // ヘルプ表示
            KeyCode::Char('?') => Message::ShowHelp,
            _ => Message::None,
        }
    }

    /// セッション詳細画面のキーマッピング
    fn session_detail_key(&self, key: KeyEvent) -> Message {
        match key.code {
            // Esc または q で一覧に戻る
            KeyCode::Esc | KeyCode::Char('q') => Message::BackToList,
            // 上にスクロール
            KeyCode::Char('k') | KeyCode::Up => Message::ScrollUp,
            // 下にスクロール
            KeyCode::Char('j') | KeyCode::Down => Message::ScrollDown,
            // エクスポートダイアログ表示
            KeyCode::Char('e') => Message::StartExport,
            // ヘルプ表示
            KeyCode::Char('?') => Message::ShowHelp,
            _ => Message::None,
        }
    }

    /// 検索モードのキーマッピング
    fn search_mode_key(&self, key: KeyEvent) -> Message {
        match key.code {
            KeyCode::Esc => Message::CancelSearch,
            KeyCode::Enter => Message::ConfirmSearch,
            KeyCode::Backspace => Message::SearchBackspace,
            KeyCode::Char(c) => Message::SearchInput(c),
            _ => Message::None,
        }
    }

    /// フィルタモードのキーマッピング
    fn filter_mode_key(&self, key: KeyEvent) -> Message {
        match key.code {
            KeyCode::Tab => Message::FilterNextField,
            KeyCode::Char('j') | KeyCode::Down => Message::FilterDatePresetNext,
            KeyCode::Char('k') | KeyCode::Up => Message::FilterDatePresetPrev,
            KeyCode::Enter => Message::ApplyFilter,
            KeyCode::Esc => Message::CancelFilter,
            KeyCode::Char('c') => Message::ClearFilter,
            KeyCode::Backspace => Message::FilterProjectBackspace,
            KeyCode::Char(c) => Message::FilterProjectInput(c),
            _ => Message::None,
        }
    }

    /// ヘルプモードのキーマッピング
    fn help_mode_key(&self, key: KeyEvent) -> Message {
        match key.code {
            // Esc または q または ? でヘルプを閉じる
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Message::CloseHelp,
            _ => Message::None,
        }
    }

    /// エクスポートモードのキーマッピング
    fn export_mode_key(&self, key: KeyEvent, export_status: Option<&ExportStatus>) -> Message {
        // エラー状態・Exporting 状態では Esc のみ受け付ける
        let is_exportable = matches!(export_status, Some(ExportStatus::Selecting));

        match key.code {
            // Esc でキャンセル（常に有効）
            KeyCode::Esc | KeyCode::Char('q') => Message::CancelExport,
            // 以下は Selecting 状態のみ有効
            // Tab または j/k で形式切り替え
            KeyCode::Tab
            | KeyCode::Char('j')
            | KeyCode::Char('k')
            | KeyCode::Left
            | KeyCode::Right
                if is_exportable =>
            {
                Message::ToggleExportFormat
            }
            // Enter でエクスポート実行
            KeyCode::Enter if is_exportable => Message::ConfirmExport,
            _ => Message::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_new() {
        let handler = EventHandler::new();
        assert_eq!(handler.timeout, Duration::from_millis(100));
    }

    #[test]
    fn test_session_list_quit() {
        let handler = EventHandler::new();

        // q キー
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::Quit
        ));

        // Esc キーはフィルタクリア（フィルタがなければ終了）
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::ClearFilter
        ));

        // Ctrl+C
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::Quit
        ));
    }

    #[test]
    fn test_session_list_move_up() {
        let handler = EventHandler::new();

        // k キー
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::MoveUp
        ));

        // 上矢印
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::MoveUp
        ));
    }

    #[test]
    fn test_session_list_move_down() {
        let handler = EventHandler::new();

        // j キー
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::MoveDown
        ));

        // 下矢印
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::MoveDown
        ));
    }

    #[test]
    fn test_session_list_enter_detail() {
        let handler = EventHandler::new();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::EnterDetail
        ));
    }

    #[test]
    fn test_session_detail_back_to_list() {
        let handler = EventHandler::new();

        // Esc キー
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::BackToList
        ));

        // q キー
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::BackToList
        ));
    }

    #[test]
    fn test_session_detail_ctrl_c_quit() {
        let handler = EventHandler::new();

        // Ctrl+C は詳細画面でも終了
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::Quit
        ));
    }

    #[test]
    fn test_session_list_unknown() {
        let handler = EventHandler::new();

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::None
        ));
    }

    #[test]
    fn test_session_detail_scroll_up() {
        let handler = EventHandler::new();

        // k キー
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::ScrollUp
        ));

        // 上矢印
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::ScrollUp
        ));
    }

    #[test]
    fn test_session_detail_scroll_down() {
        let handler = EventHandler::new();

        // j キー
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::ScrollDown
        ));

        // 下矢印
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionDetail, None),
            Message::ScrollDown
        ));
    }

    #[test]
    fn test_session_list_start_search() {
        let handler = EventHandler::new();

        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::StartSearch
        ));
    }

    #[test]
    fn test_session_list_start_filter() {
        let handler = EventHandler::new();

        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::SessionList, None),
            Message::StartFilter
        ));
    }

    #[test]
    fn test_search_mode() {
        let handler = EventHandler::new();

        // 文字入力
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Search, None),
            Message::SearchInput('a')
        ));

        // バックスペース
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Search, None),
            Message::SearchBackspace
        ));

        // Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Search, None),
            Message::ConfirmSearch
        ));

        // Esc
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Search, None),
            Message::CancelSearch
        ));
    }

    #[test]
    fn test_filter_mode() {
        let handler = EventHandler::new();

        // Tab
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::FilterNextField
        ));

        // j/↓ for next preset
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::FilterDatePresetNext
        ));

        // k/↑ for prev preset
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::FilterDatePresetPrev
        ));

        // Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::ApplyFilter
        ));

        // Esc
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::CancelFilter
        ));

        // c for clear
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());
        assert!(matches!(
            handler.key_to_message(key, ViewMode::Filter, None),
            Message::ClearFilter
        ));
    }
}
