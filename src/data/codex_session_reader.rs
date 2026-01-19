use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::KathaError;
use crate::domain::{ContentBlock, Message, MessageContent, Session, SessionEntry};

#[derive(Debug, Clone)]
pub struct CodexSessionInfo {
    pub session_id: String,
    pub path: PathBuf,
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexSessionLine {
    pub timestamp: Option<String>,
    #[serde(rename = "type")]
    pub line_type: String,
    pub payload: Value,
}

pub struct CodexSessionReader;

impl CodexSessionReader {
    pub fn build_session_index<P: AsRef<Path>>(
        sessions_dir: P,
    ) -> Result<HashMap<String, CodexSessionInfo>, KathaError> {
        let mut files = Vec::new();
        collect_jsonl_files(sessions_dir.as_ref(), &mut files)?;

        let mut index = HashMap::new();
        for path in files {
            if let Some(info) = Self::session_info_from_file(&path)? {
                index.insert(info.session_id.clone(), info);
            }
        }

        Ok(index)
    }

    fn session_info_from_file(path: &Path) -> Result<Option<CodexSessionInfo>, KathaError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<CodexSessionLine>(&line) {
                Ok(line) => {
                    if line.line_type != "session_meta" {
                        continue;
                    }
                    let session_id = line
                        .payload
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let cwd = line
                        .payload
                        .get("cwd")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);

                    if let Some(session_id) = session_id {
                        return Ok(Some(CodexSessionInfo {
                            session_id,
                            path: path.to_path_buf(),
                            cwd,
                        }));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Codex session meta line {} in {}: {}",
                        line_num + 1,
                        path.display(),
                        e
                    );
                    break;
                }
            }
        }

        Ok(None)
    }

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

            let parsed = match serde_json::from_str::<CodexSessionLine>(&line) {
                Ok(line) => line,
                Err(e) => {
                    eprintln!(
                        "Warning: Codex session line {} in {}: {}",
                        line_num + 1,
                        path.as_ref().display(),
                        e
                    );
                    continue;
                }
            };

            if parsed.line_type != "response_item" {
                continue;
            }

            let payload_type = parsed.payload.get("type").and_then(|v| v.as_str());
            if payload_type != Some("message") {
                continue;
            }

            let role = parsed
                .payload
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if role != "user" && role != "assistant" {
                continue;
            }

            let content = parsed.payload.get("content").and_then(|v| v.as_array());
            let mut blocks = Vec::new();
            if let Some(items) = content {
                for item in items {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        blocks.push(ContentBlock::Text {
                            text: text.to_string(),
                        });
                    }
                }
            }

            if blocks.is_empty() {
                continue;
            }

            let message = Message {
                role: role.to_string(),
                content: MessageContent::Blocks(blocks),
                model: None,
                id: None,
                stop_reason: None,
                usage: None,
            };

            entries.push(SessionEntry {
                entry_type: role.to_string(),
                message: Some(message),
                timestamp: parsed.timestamp.clone(),
                ..Default::default()
            });
        }

        Ok(entries)
    }
}

fn collect_jsonl_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), KathaError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "jsonl") {
            files.push(path);
        }
    }

    Ok(())
}
