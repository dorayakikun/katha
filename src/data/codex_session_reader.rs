use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;
use tracing::warn;

use crate::KathaError;
use crate::domain::message::Usage;
use crate::domain::{ContentBlock, Message, MessageContent, Session, SessionEntry};

#[derive(Debug, Clone)]
pub struct CodexSessionInfo {
    pub session_id: String,
    pub path: PathBuf,
    pub cwd: Option<String>,
}

#[derive(Debug)]
struct CodexSessionLine {
    pub timestamp: Option<String>,
    pub line_type: Option<String>,
    pub payload: Value,
}

fn parse_codex_line(line: &str) -> Result<CodexSessionLine, serde_json::Error> {
    let value: Value = serde_json::from_str(line)?;
    let timestamp = value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let mut line_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let payload = value
        .get("payload")
        .cloned()
        .unwrap_or_else(|| value.clone());

    if line_type.is_none() && payload.get("id").and_then(|v| v.as_str()).is_some() {
        line_type = Some("session_meta".to_string());
    }

    Ok(CodexSessionLine {
        timestamp,
        line_type,
        payload,
    })
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

            match parse_codex_line(&line) {
                Ok(line) => {
                    if line.line_type.as_deref() != Some("session_meta") {
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
                    warn!(
                        "Codex session meta line {} in {}: {}",
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
        let mut entries: Vec<SessionEntry> = Vec::new();
        let mut last_assistant_index: Option<usize> = None;
        let mut last_model: Option<String> = None;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let parsed = match parse_codex_line(&line) {
                Ok(line) => line,
                Err(e) => {
                    warn!(
                        "Codex session line {} in {}: {}",
                        line_num + 1,
                        path.as_ref().display(),
                        e
                    );
                    continue;
                }
            };

            if parsed.line_type.as_deref() == Some("event_msg") {
                if let Some(usage) = parse_token_count_usage(&parsed.payload) {
                    if let Some(idx) = last_assistant_index {
                        if let Some(entry) = entries.get_mut(idx) {
                            if let Some(message) = entry.message.as_mut() {
                                if message.usage.is_none() {
                                    message.usage = Some(usage);
                                }
                            }
                        }
                    }
                }
                continue;
            }

            if parsed.line_type.as_deref() == Some("turn_context") {
                last_model = parsed
                    .payload
                    .get("model")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                continue;
            }

            if parsed.line_type.as_deref() != Some("response_item") {
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

            let mut model = parsed
                .payload
                .get("model")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if model.is_none() {
                model = last_model.clone();
            }
            let usage = parse_usage(&parsed.payload);

            let message = Message {
                role: role.to_string(),
                content: MessageContent::Blocks(blocks),
                model,
                id: None,
                stop_reason: None,
                usage,
            };

            entries.push(SessionEntry {
                entry_type: Some(role.to_string()),
                message: Some(message),
                timestamp: parsed.timestamp.clone(),
                ..Default::default()
            });

            if role == "assistant" {
                last_assistant_index = Some(entries.len() - 1);
            }
        }

        backfill_assistant_models(&mut entries);
        Ok(entries)
    }
}

fn parse_usage(payload: &Value) -> Option<Usage> {
    let usage = payload.get("usage")?;
    let input_tokens = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("prompt_tokens").and_then(|v| v.as_u64()));
    let output_tokens = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("completion_tokens").and_then(|v| v.as_u64()));
    let cache_creation_input_tokens = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64());
    let cache_read_input_tokens = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64());

    if input_tokens.is_none()
        && output_tokens.is_none()
        && cache_creation_input_tokens.is_none()
        && cache_read_input_tokens.is_none()
    {
        return None;
    }

    Some(Usage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
    })
}

fn parse_token_count_usage(payload: &Value) -> Option<Usage> {
    if payload.get("type").and_then(|v| v.as_str()) != Some("token_count") {
        return None;
    }
    let info = payload.get("info")?;
    let last = info.get("last_token_usage")?;
    let input_tokens = last.get("input_tokens").and_then(|v| v.as_u64());
    let output_tokens = last.get("output_tokens").and_then(|v| v.as_u64());
    let cached_input_tokens = last.get("cached_input_tokens").and_then(|v| v.as_u64());

    if input_tokens.is_none() && output_tokens.is_none() && cached_input_tokens.is_none() {
        return None;
    }

    Some(Usage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: cached_input_tokens,
    })
}

fn backfill_assistant_models(entries: &mut [SessionEntry]) {
    // Forward fill with the last seen assistant model.
    let mut last_model: Option<String> = None;
    for entry in entries.iter_mut() {
        let Some(message) = entry.message.as_mut() else { continue };
        if message.role != "assistant" {
            continue;
        }
        if let Some(model) = message.model.clone() {
            last_model = Some(model);
        } else if let Some(model) = last_model.clone() {
            message.model = Some(model);
        }
    }

    // Backward fill any remaining gaps using the next seen model.
    let mut next_model: Option<String> = None;
    for entry in entries.iter_mut().rev() {
        let Some(message) = entry.message.as_mut() else { continue };
        if message.role != "assistant" {
            continue;
        }
        if let Some(model) = message.model.clone() {
            next_model = Some(model);
        } else if let Some(model) = next_model.clone() {
            message.model = Some(model);
        }
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
