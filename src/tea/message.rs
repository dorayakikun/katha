use std::path::PathBuf;

use crate::domain::Session;
use crate::export::ExportFormat;

/// TEA アーキテクチャのメッセージ型
/// UI イベントを表現する
#[derive(Debug, Clone)]
pub enum Message {
    /// 初期化完了
    Initialized,
    /// セッション選択
    SelectSession(usize),
    /// 上に移動
    MoveUp,
    /// 下に移動
    MoveDown,
    /// 詳細画面に遷移
    EnterDetail,
    /// 一覧画面に戻る
    BackToList,
    /// 上にスクロール
    ScrollUp(usize),
    /// 下にスクロール
    ScrollDown(usize),
    /// 選択中メッセージをコピー
    CopySelectedMessage,
    /// 選択中メッセージをメタ情報付きでコピー
    CopySelectedMessageWithMeta,
    /// コスト表示通貨を切り替え
    ToggleCurrency,
    /// セッション読み込み完了
    SessionLoaded(Session),
    /// セッション読み込みエラー
    SessionLoadFailed(String),
    /// 終了
    Quit,
    /// 何もしない
    None,

    // === 検索関連 ===
    /// 検索モード開始
    StartSearch,
    /// 検索モードキャンセル
    CancelSearch,
    /// 検索入力
    SearchInput(char),
    /// 検索バックスペース
    SearchBackspace,
    /// 検索確定
    ConfirmSearch,

    // === フィルタ関連 ===
    /// フィルタモード開始
    StartFilter,
    /// フィルタモードキャンセル
    CancelFilter,
    /// フィルタ適用
    ApplyFilter,
    /// フィルタクリア
    ClearFilter,
    /// 次のフィールドへ移動
    FilterNextField,
    /// 日付プリセット次へ
    FilterDatePresetNext,
    /// 日付プリセット前へ
    FilterDatePresetPrev,
    /// プロジェクト入力
    FilterProjectInput(char),
    /// プロジェクトバックスペース
    FilterProjectBackspace,

    // === ヘルプ関連 ===
    /// ヘルプ表示
    ShowHelp,
    /// ヘルプを閉じる
    CloseHelp,

    // === エクスポート関連 ===
    /// エクスポートダイアログ表示
    StartExport,
    /// エクスポート形式選択
    SelectExportFormat(ExportFormat),
    /// エクスポート形式切り替え
    ToggleExportFormat,
    /// エクスポート実行
    ConfirmExport,
    /// エクスポートキャンセル
    CancelExport,
    /// エクスポート完了
    ExportCompleted(PathBuf),
    /// エクスポート失敗
    ExportFailed(String),

    // === エラー関連 ===
    /// エラーメッセージを表示
    ShowError(String),
    /// エラーメッセージをクリア
    ClearError,

    // === ツリー操作関連 ===
    /// プロジェクトの展開/折りたたみ切り替え
    ToggleProject(String),
    /// 選択中のプロジェクトを展開
    ExpandCurrentProject,
    /// 選択中のプロジェクトを折りたたみ
    CollapseCurrentProject,
    /// すべてのプロジェクトを展開
    ExpandAll,
    /// すべてのプロジェクトを折りたたみ
    CollapseAll,
}
