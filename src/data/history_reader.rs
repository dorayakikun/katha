use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::KathaError;
use crate::domain::HistoryEntry;

pub struct HistoryReader;

impl HistoryReader {
    /// 全エントリを読み込み（新しい順）
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
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Line {}: {}", line_num + 1, e);
                }
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// ユニークセッション ID
    pub fn unique_session_ids<P: AsRef<Path>>(path: P) -> Result<Vec<String>, KathaError> {
        let entries = Self::read_all(path)?;
        let mut ids: Vec<String> = entries.into_iter().map(|e| e.session_id).collect();
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
                .entry(entry.project.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        Ok(grouped)
    }
}
