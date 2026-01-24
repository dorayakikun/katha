use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use tracing::debug;

use crate::KathaError;
use crate::domain::HistoryEntry;

pub struct HistoryReader;

impl HistoryReader {
    /// 全エントリを読み込み（新しい順）
    /// 緩いスキーマでパースし、無効なエントリはスキップ
    pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<HistoryEntry>, KathaError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<HistoryEntry>(&line) {
                Ok(entry) => {
                    if entry.is_valid() {
                        entries.push(entry);
                    } else {
                        debug!("Line {}: skipped entry (missing session_id)", line_num + 1);
                    }
                }
                Err(e) => {
                    debug!("Line {}: parse error: {}", line_num + 1, e);
                }
            }
        }

        entries.sort_by(|a, b| b.datetime().cmp(&a.datetime()));
        Ok(entries)
    }

    /// ユニークセッション ID
    pub fn unique_session_ids<P: AsRef<Path>>(path: P) -> Result<Vec<String>, KathaError> {
        let entries = Self::read_all(path)?;
        let mut ids: Vec<String> = entries
            .into_iter()
            .filter_map(|e| e.session_id)
            .collect();
        ids.dedup();
        Ok(ids)
    }

    /// プロジェクト別グループ化
    pub fn group_by_project<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<String, Vec<HistoryEntry>>, KathaError> {
        let entries = Self::read_all(path)?;
        let mut grouped = HashMap::new();

        for entry in entries {
            grouped
                .entry(entry.project().to_string())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        Ok(grouped)
    }
}
