use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use crate::KathaError;

#[derive(Debug, Clone, Deserialize)]
pub struct CodexHistoryEntry {
    pub session_id: String,
    pub ts: i64,
    pub text: String,
}

pub struct CodexHistoryReader;

impl CodexHistoryReader {
    /// 全エントリを読み込み（新しい順）
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
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Codex history line {}: {}", line_num + 1, e);
                }
            }
        }

        entries.sort_by(|a, b| b.ts.cmp(&a.ts));
        Ok(entries)
    }
}
