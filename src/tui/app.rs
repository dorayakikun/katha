use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};

use chrono::{DateTime, TimeZone, Utc};
use crossterm::{execute, clipboard::CopyToClipboard};
use ratatui::layout::{Constraint, Layout, Rect};
use tracing::{debug, trace};

use crate::KathaError;
use crate::config::{ClaudePaths, CodexPaths};
use crate::data::{CodexHistoryReader, CodexSessionInfo, CodexSessionReader, HistoryReader, SessionReader};
use crate::export::{
    ExportFormat, Exporter, JsonExporter, MarkdownExporter, generate_filename, write_to_file,
};
use crate::tea::{
    ExportStatus, Message, Model, ProjectGroup, SessionListItem, SessionSource, TreeNodeKind,
    ViewMode, update,
};
use crate::tui::{EventHandler, Terminal};
use crate::views::{render_export_dialog, render_help, render_session_detail, render_session_list};

#[derive(Debug, Clone)]
struct HistoryItem {
    session_id: String,
    project_path: String,
    display: String,
    timestamp: i64,
    source: SessionSource,
}

fn datetime_from_millis(timestamp_ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(timestamp_ms)
        .single()
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

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
    /// Codex パス設定
    codex_paths: Option<CodexPaths>,
    /// Codex セッション一覧
    codex_sessions: HashMap<String, CodexSessionInfo>,
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
            codex_paths: None,
            codex_sessions: HashMap::new(),
            async_tx,
            async_rx,
        })
    }

    /// セッションを読み込み
    pub fn load_sessions(&mut self) -> Result<(), KathaError> {
        let paths = ClaudePaths::new()?;
        let mut history_items: HashMap<String, Vec<HistoryItem>> = HashMap::new();

        if paths.history_exists() {
            let entries = HistoryReader::read_all(&paths.history_file)?;
            for entry in entries {
                history_items
                    .entry(entry.project.clone())
                    .or_insert_with(Vec::new)
                    .push(HistoryItem {
                        session_id: entry.session_id.clone(),
                        project_path: entry.project.clone(),
                        display: entry.display.clone(),
                        timestamp: entry.timestamp,
                        source: SessionSource::Claude,
                    });
            }
        }

        self.paths = Some(paths);
        self.codex_paths = None;
        self.codex_sessions.clear();

        if let Ok(codex_paths) = CodexPaths::new() {
            self.codex_sessions =
                CodexSessionReader::build_session_index(&codex_paths.sessions_dir)?;

            if codex_paths.history_exists() {
                let entries = CodexHistoryReader::read_all(&codex_paths.history_file)?;
                for entry in entries {
                    let project_path = self
                        .codex_sessions
                        .get(&entry.session_id)
                        .and_then(|info| info.cwd.clone())
                        .unwrap_or_else(|| "Codex".to_string());
                    history_items
                        .entry(project_path.clone())
                        .or_insert_with(Vec::new)
                        .push(HistoryItem {
                            session_id: entry.session_id.clone(),
                            project_path,
                            display: entry.text.clone(),
                            timestamp: entry.ts * 1000,
                            source: SessionSource::Codex,
                        });
                }
            }

            self.codex_paths = Some(codex_paths);
        }

        if history_items.is_empty() {
            self.model = Model::new();
            update(&mut self.model, Message::Initialized);
            return Ok(());
        }

        // ProjectGroup に変換（全セッションを含む）
        let mut project_groups: Vec<ProjectGroup> = Vec::new();

        for (project, mut entries) in history_items {
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            let project_name = project.rsplit('/').next().unwrap_or(&project).to_string();

            // 全エントリを SessionListItem に変換
            // history.jsonl is newest-first; keep only the latest entry per session.
            let mut seen_session_keys: HashSet<(SessionSource, String)> = HashSet::new();
            let sessions: Vec<SessionListItem> = entries
                .iter()
                .filter(|entry| {
                    seen_session_keys.insert((entry.source, entry.session_id.clone()))
                })
                .map(|entry| {
                    let datetime = datetime_from_millis(entry.timestamp);
                    let formatted_time = datetime.format("%Y-%m-%d %H:%M").to_string();

                    SessionListItem {
                        session_id: entry.session_id.clone(),
                        source: entry.source,
                        project_name: project_name.clone(),
                        project_path: entry.project_path.clone(),
                        latest_user_message: entry.display.clone(),
                        formatted_time,
                        datetime,
                    }
                })
                .collect();

            if !sessions.is_empty() {
                project_groups.push(ProjectGroup {
                    project_path: project.clone(),
                    project_name,
                    sessions,
                });
            }
        }

        // 各プロジェクトの最新セッションの時刻でソート（新しい順）
        project_groups.sort_by(|a, b| {
            let a_latest = a.sessions.first().map(|s| &s.datetime);
            let b_latest = b.sessions.first().map(|s| &s.datetime);
            b_latest.cmp(&a_latest)
        });

        self.model = Model::new().with_project_groups(project_groups);
        update(&mut self.model, Message::Initialized);

        Ok(())
    }

    fn update_detail_viewport(&mut self, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);
        let content = layout[1];
        let width = content.width.saturating_sub(2) as usize;
        let height = content.height.saturating_sub(2) as usize;
        self.model.set_detail_viewport(width, height);
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
        // ツリーアイテムからセッション情報を取得
        let selected = if let Some(item) = self.model.selected_tree_item() {
            // Session ノードの場合のみ
            item.session.as_ref()?
        } else {
            // 旧方式（互換性のため）
            self.model.selected_session()?
        };

        match selected.source {
            SessionSource::Claude => {
                let paths = self.paths.as_ref()?;
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
            SessionSource::Codex => {
                let info = match self.codex_sessions.get(&selected.session_id) {
                    Some(info) => info,
                    None => {
                        return Some(Message::SessionLoadFailed(format!(
                            "Codex session not found: {}",
                            selected.session_id
                        )));
                    }
                };

                if !info.path.exists() {
                    return Some(Message::SessionLoadFailed(format!(
                        "Codex session file not found: {}",
                        info.path.display()
                    )));
                }

                match CodexSessionReader::read_session(
                    &info.path,
                    &selected.session_id,
                    &selected.project_path,
                ) {
                    Ok(session) => Some(Message::SessionLoaded(session)),
                    Err(e) => Some(Message::SessionLoadFailed(e.to_string())),
                }
            }
        }
    }

    fn copy_selected_message(&self, with_meta: bool) -> Result<(), String> {
        let entry = self.selected_detail_entry()?;
        let text = entry
            .display_text()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "Selected message has no text".to_string())?;

        let content = if with_meta {
            let role = if entry.is_user() { "user" } else { "assistant" };
            let timestamp = entry
                .datetime()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            if timestamp.is_empty() {
                format!("{}\n{}", role, text)
            } else {
                format!("{} {}\n{}", role, timestamp, text)
            }
        } else {
            text
        };

        execute!(
            std::io::stdout(),
            CopyToClipboard::to_clipboard_from(content)
        )
        .map_err(|e| format!("Clipboard copy failed: {}", e))?;

        Ok(())
    }

    fn selected_detail_entry(&self) -> Result<&crate::domain::SessionEntry, String> {
        self.model
            .detail_entry_for_cursor()
            .ok_or_else(|| "No message selected".to_string())
    }


    /// メインループ
    pub fn run(&mut self) -> Result<(), KathaError> {
        debug!(
            "Entering main loop, should_quit: {}, session_count: {}",
            self.model.should_quit,
            self.model.sessions.len()
        );

        loop {
            // 1. まず非同期メッセージを受信（描画前に処理することで即座に画面更新）
            while let Ok(async_msg) = self.async_rx.try_recv() {
                trace!("Received async message: {:?}", async_msg);
                update(&mut self.model, async_msg);
            }

            // 2. view_mode に応じた描画
            let view_mode = self.model.view_mode;
            let area = {
                let terminal = self.terminal.inner();
                let size = terminal
                    .size()
                    .map_err(|e| KathaError::Terminal(e.to_string()))?;
                Rect::new(0, 0, size.width, size.height)
            };
            let needs_detail = matches!(view_mode, ViewMode::SessionDetail)
                || (matches!(view_mode, ViewMode::Help | ViewMode::Export)
                    && matches!(self.model.previous_view_mode, ViewMode::SessionDetail));
            if needs_detail {
                self.update_detail_viewport(area);
            }

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
            trace!("Waiting for event...");
            let msg = self
                .event_handler
                .poll(view_mode, self.model.export_status.as_ref())?;
            trace!("Event message: {:?}", msg);

            // キー入力があったらエラーメッセージをクリア（セッション一覧画面のみ）
            if !matches!(msg, Message::None) && self.model.error_message.is_some() {
                update(&mut self.model, Message::ClearError);
            }

            // EnterDetail の場合
            if matches!(msg, Message::EnterDetail) {
                // Session ノードの場合のみセッションを読み込む
                let is_session_node = self
                    .model
                    .selected_tree_item()
                    .map(|item| item.kind == TreeNodeKind::Session)
                    .unwrap_or(false);

                update(&mut self.model, msg);

                // Session ノードの場合はセッション読み込みを実行
                if is_session_node {
                    if let Some(load_msg) = self.load_current_session() {
                        update(&mut self.model, load_msg);
                    }
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
            } else if matches!(
                msg,
                Message::CopySelectedMessage | Message::CopySelectedMessageWithMeta
            ) {
                let with_meta = matches!(msg, Message::CopySelectedMessageWithMeta);
                if let Err(error) = self.copy_selected_message(with_meta) {
                    update(&mut self.model, Message::ShowError(error));
                }
            } else {
                update(&mut self.model, msg);
            }

            // 終了判定
            if self.model.should_quit {
                debug!("should_quit is true, exiting loop");
                break;
            }
        }

        debug!("Exiting main loop");
        // ターミナル復元
        self.terminal.restore()?;

        Ok(())
    }
}
