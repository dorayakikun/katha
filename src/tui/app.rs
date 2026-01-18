use std::sync::mpsc::{self, Receiver, Sender};

use crate::KathaError;
use crate::config::ClaudePaths;
use crate::data::{HistoryReader, SessionReader};
use crate::export::{
    ExportFormat, Exporter, JsonExporter, MarkdownExporter, generate_filename, write_to_file,
};
use crate::tea::{ExportStatus, Message, Model, SessionListItem, ViewMode, update};
use crate::tui::{EventHandler, Terminal};
use crate::views::{render_export_dialog, render_help, render_session_detail, render_session_list};

/// アプリケーション
pub struct App {
    /// TEA Model
    model: Model,
    /// ターミナル
    terminal: Terminal,
    /// イベントハンドラ
    event_handler: EventHandler,
    /// Claude パス設定
    paths: Option<ClaudePaths>,
    /// 非同期メッセージ送信用
    async_tx: Sender<Message>,
    /// 非同期メッセージ受信用
    async_rx: Receiver<Message>,
}

impl App {
    /// 新規作成
    pub fn new() -> Result<Self, KathaError> {
        let terminal = Terminal::new()?;
        let event_handler = EventHandler::new();
        let model = Model::new();
        let (async_tx, async_rx) = mpsc::channel();

        Ok(Self {
            model,
            terminal,
            event_handler,
            paths: None,
            async_tx,
            async_rx,
        })
    }

    /// セッションを読み込み
    pub fn load_sessions(&mut self) -> Result<(), KathaError> {
        let paths = ClaudePaths::new()?;

        if !paths.history_exists() {
            // history.jsonl が存在しない場合は空のリストで続行
            self.paths = Some(paths);
            return Ok(());
        }

        // プロジェクトごとにグループ化されたエントリを取得
        let projects = HistoryReader::group_by_project(&paths.history_file)?;

        // SessionListItem に変換
        let mut sessions: Vec<SessionListItem> = Vec::new();

        for (project, entries) in projects {
            // 各プロジェクトの最新エントリを使用
            if let Some(entry) = entries.first() {
                let project_name = project.rsplit('/').next().unwrap_or(&project).to_string();

                let datetime = entry.datetime();
                let formatted_time = datetime.format("%Y-%m-%d %H:%M").to_string();

                let display = entry.display.clone();

                sessions.push(SessionListItem {
                    session_id: entry.session_id.clone(),
                    project_name,
                    project_path: project.clone(),
                    display,
                    formatted_time,
                    datetime,
                });
            }
        }

        // 時刻でソート（新しい順）
        sessions.sort_by(|a, b| b.formatted_time.cmp(&a.formatted_time));

        self.paths = Some(paths);
        self.model = Model::new().with_sessions(sessions);
        update(&mut self.model, Message::Initialized);

        Ok(())
    }

    /// エクスポートを別スレッドで開始
    fn start_export(&self) {
        let session = match self.model.current_session.clone() {
            Some(s) => s,
            None => return,
        };

        let format = self.model.export_format;
        let tx = self.async_tx.clone();

        std::thread::spawn(move || {
            let content = match format {
                ExportFormat::Markdown => MarkdownExporter::new().export(&session),
                ExportFormat::Json => JsonExporter::new().export(&session),
            };

            let filename = generate_filename(&session, format);

            let msg = match write_to_file(&content, &filename, None) {
                Ok(path) => Message::ExportCompleted(path),
                Err(e) => Message::ExportFailed(e.to_string()),
            };

            // エラーは無視（アプリ終了中の可能性があるため）
            let _ = tx.send(msg);
        });
    }

    /// 選択中のセッションを読み込み
    fn load_current_session(&self) -> Option<Message> {
        let paths = self.paths.as_ref()?;
        let selected = self.model.selected_session()?;

        let session_file = SessionReader::session_file_path(
            &paths.projects_dir,
            &selected.project_path,
            &selected.session_id,
        );

        if !session_file.exists() {
            return Some(Message::SessionLoadFailed(format!(
                "Session file not found: {}",
                session_file.display()
            )));
        }

        match SessionReader::read_session(
            &session_file,
            &selected.session_id,
            &selected.project_path,
        ) {
            Ok(session) => Some(Message::SessionLoaded(session)),
            Err(e) => Some(Message::SessionLoadFailed(e.to_string())),
        }
    }

    /// メインループ
    pub fn run(&mut self) -> Result<(), KathaError> {
        loop {
            // 1. まず非同期メッセージを受信（描画前に処理することで即座に画面更新）
            while let Ok(async_msg) = self.async_rx.try_recv() {
                update(&mut self.model, async_msg);
            }

            // 2. view_mode に応じた描画
            let view_mode = self.model.view_mode;
            self.terminal
                .inner()
                .draw(|frame| match view_mode {
                    ViewMode::SessionList | ViewMode::Search | ViewMode::Filter => {
                        render_session_list(frame, &self.model);
                    }
                    ViewMode::SessionDetail => {
                        render_session_detail(frame, &self.model);
                    }
                    ViewMode::Help => {
                        // 元のビューを描画してからヘルプをオーバーレイ
                        match self.model.previous_view_mode {
                            ViewMode::SessionList | ViewMode::Search | ViewMode::Filter => {
                                render_session_list(frame, &self.model);
                            }
                            ViewMode::SessionDetail => {
                                render_session_detail(frame, &self.model);
                            }
                            _ => {}
                        }
                        render_help(frame, &self.model);
                    }
                    ViewMode::Export => {
                        // 元のビューを描画してからエクスポートダイアログをオーバーレイ
                        match self.model.previous_view_mode {
                            ViewMode::SessionList | ViewMode::Search | ViewMode::Filter => {
                                render_session_list(frame, &self.model);
                            }
                            ViewMode::SessionDetail => {
                                render_session_detail(frame, &self.model);
                            }
                            _ => {}
                        }
                        render_export_dialog(frame, &self.model);
                    }
                })
                .map_err(|e| KathaError::Terminal(e.to_string()))?;

            // 3. イベント処理
            let msg = self
                .event_handler
                .poll(view_mode, self.model.export_status.as_ref())?;

            // キー入力があったらエラーメッセージをクリア（セッション一覧画面のみ）
            if !matches!(msg, Message::None)
                && matches!(view_mode, ViewMode::SessionList)
                && self.model.error_message.is_some()
            {
                update(&mut self.model, Message::ClearError);
            }

            // EnterDetail の場合はセッション読み込みを実行
            if matches!(msg, Message::EnterDetail) {
                update(&mut self.model, msg);

                // セッション読み込み
                if let Some(load_msg) = self.load_current_session() {
                    update(&mut self.model, load_msg);
                }
            } else if matches!(msg, Message::ConfirmExport) {
                // current_session が存在し、Selecting 状態の場合のみ export を開始
                if self.model.current_session.is_some()
                    && matches!(self.model.export_status, Some(ExportStatus::Selecting))
                {
                    // Exporting 状態に設定（次のループで描画される）
                    update(&mut self.model, msg);
                    // 別スレッドでエクスポート開始
                    self.start_export();
                }
                // current_session が None または Selecting 以外の状態では何もしない
            } else if matches!(msg, Message::StartExport) {
                // セッションが読み込まれていない場合は先に読み込む
                if self.model.current_session.is_none() {
                    if let Some(load_msg) = self.load_current_session() {
                        update(&mut self.model, load_msg.clone());
                        // 読み込み失敗時はエラーメッセージを表示（ダイアログは開かない）
                        if let Message::SessionLoadFailed(error) = load_msg {
                            update(
                                &mut self.model,
                                Message::ShowError(format!("Failed to load session: {}", error)),
                            );
                            // ここで処理を終了（ダイアログを開かない）
                        } else {
                            // セッションが正常に読み込まれた場合のみエクスポートダイアログを開く
                            update(&mut self.model, msg);
                        }
                    }
                } else {
                    // 既にセッションが読み込まれている場合はダイアログを開く
                    update(&mut self.model, msg);
                }
            } else {
                update(&mut self.model, msg);
            }

            // 終了判定
            if self.model.should_quit {
                break;
            }
        }

        // ターミナル復元
        self.terminal.restore()?;

        Ok(())
    }
}
