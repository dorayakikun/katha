use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;
use tracing::warn;

use crate::KathaError;

/// Codex の history.jsonl エントリ
/// 緩いスキーマ設計：すべてのフィールドを Option にしてパースエラーを防ぐ
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CodexHistoryEntry {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub text: Option<String>,
}

impl CodexHistoryEntry {
    /// エントリが有効かどうか（最低限必要なフィールドが存在するか）
    pub fn is_valid(&self) -> bool {
        self.session_id.is_some()
    }

    /// セッション ID を取得
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// タイムスタンプを取得（デフォルト: 0）
    pub fn ts(&self) -> i64 {
        self.ts.unwrap_or(0)
    }

    /// テキストを取得（デフォルト: ""）
    pub fn text(&self) -> &str {
        self.text.as_deref().unwrap_or("")
    }
}

pub struct CodexHistoryReader;

impl CodexHistoryReader {
    /// 全エントリを読み込み（新しい順）
    /// 緩いスキーマでパースし、無効なエントリはスキップ
    pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<CodexHistoryEntry>, KathaError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<CodexHistoryEntry>(&line) {
                Ok(entry) => {
                    if entry.is_valid() {
                        entries.push(entry);
                    } else {
                        warn!("Codex history line {}: invalid entry (missing required fields)", line_num + 1);
                    }
                }
                Err(e) => {
                    warn!("Codex history line {}: {}", line_num + 1, e);
                }
            }
        }

        entries.sort_by(|a, b| b.ts().cmp(&a.ts()));
        Ok(entries)
    }
}
