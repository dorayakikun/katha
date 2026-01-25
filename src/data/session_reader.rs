use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use tracing::warn;

use crate::KathaError;
use crate::config::ClaudePaths;
use crate::domain::{Session, SessionEntry};

pub struct SessionReader;

impl SessionReader {
    /// セッションを読み込み
    pub fn read_session<P: AsRef<Path>>(
        path: P,
        session_id: &str,
        project: &str,
    ) -> Result<Session, KathaError> {
        let entries = Self::read_entries(path)?;
        Ok(Session::from_entries(
            session_id.to_string(),
            project.to_string(),
            entries,
        ))
    }

    /// エントリを読み込み
    pub fn read_entries<P: AsRef<Path>>(path: P) -> Result<Vec<SessionEntry>, KathaError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<SessionEntry>(&line) {
                Ok(entry) => {
                    // entry_type が None のエントリはスキップ
                    if entry.entry_type.is_none() {
                        warn!("Session line {}: missing type field", line_num + 1);
                        continue;
                    }
                    // file-history-snapshot はスキップ
                    if entry.entry_type.as_deref() != Some("file-history-snapshot") {
                        entries.push(entry);
                    }
                }
                Err(e) => {
                    warn!("Session line {}: {}", line_num + 1, e);
                }
            }
        }
        Ok(entries)
    }

    /// プロジェクト内のセッションファイル一覧
    pub fn list_session_files<P: AsRef<Path>>(
        projects_dir: P,
        project_path: &str,
    ) -> Result<Vec<PathBuf>, KathaError> {
        let encoded = ClaudePaths::encode_project_path_for_fs(project_path);
        let project_dir = projects_dir.as_ref().join(&encoded);

        if !project_dir.exists() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&project_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                files.push(path);
            }
        }
        Ok(files)
    }

    /// セッションファイルパスを構築
    pub fn session_file_path<P: AsRef<Path>>(
        projects_dir: P,
        project_path: &str,
        session_id: &str,
    ) -> PathBuf {
        let encoded = ClaudePaths::encode_project_path_for_fs(project_path);
        projects_dir
            .as_ref()
            .join(&encoded)
            .join(format!("{}.jsonl", session_id))
    }
}
