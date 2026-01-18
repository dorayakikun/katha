use serde::Serialize;

use crate::domain::Session;

use super::Exporter;

/// JSON エクスポータ
pub struct JsonExporter {
    /// 整形出力するか
    pretty: bool,
}

impl JsonExporter {
    /// 新規作成
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// 整形出力を設定
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// エクスポート用のセッション構造体
#[derive(Serialize)]
struct ExportSession<'a> {
    id: &'a str,
    project: &'a str,
    project_name: &'a str,
    slug: Option<&'a str>,
    started_at: Option<String>,
    ended_at: Option<String>,
    message_count: usize,
    messages: Vec<ExportMessage<'a>>,
}

/// エクスポート用のメッセージ構造体
#[derive(Serialize)]
struct ExportMessage<'a> {
    role: &'a str,
    content: Option<String>,
    timestamp: Option<&'a str>,
}

impl Exporter for JsonExporter {
    fn export(&self, session: &Session) -> String {
        let messages: Vec<ExportMessage> = session
            .entries
            .iter()
            .filter(|e| e.is_user() || e.is_assistant())
            .map(|entry| ExportMessage {
                role: if entry.is_user() { "user" } else { "assistant" },
                content: entry.display_text(),
                timestamp: entry.timestamp.as_deref(),
            })
            .collect();

        let export_session = ExportSession {
            id: &session.id,
            project: &session.project,
            project_name: session.project_name(),
            slug: session.slug.as_deref(),
            started_at: session.started_at.map(|dt| dt.to_rfc3339()),
            ended_at: session.ended_at.map(|dt| dt.to_rfc3339()),
            message_count: session.message_count(),
            messages,
        };

        if self.pretty {
            serde_json::to_string_pretty(&export_session).unwrap_or_default()
        } else {
            serde_json::to_string(&export_session).unwrap_or_default()
        }
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::{Message, MessageContent};
    use crate::domain::session::SessionEntry;

    fn create_test_session() -> Session {
        let entries = vec![
            SessionEntry {
                entry_type: "user".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                message: Some(Message {
                    role: "user".to_string(),
                    content: MessageContent::Text("Hello, Claude!".to_string()),
                    model: None,
                    id: None,
                    stop_reason: None,
                    usage: None,
                }),
                ..Default::default()
            },
            SessionEntry {
                entry_type: "assistant".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                message: Some(Message {
                    role: "assistant".to_string(),
                    content: MessageContent::Text("Hello! How can I help you?".to_string()),
                    model: None,
                    id: None,
                    stop_reason: None,
                    usage: None,
                }),
                ..Default::default()
            },
        ];

        Session::from_entries(
            "test-session".to_string(),
            "/test/project".to_string(),
            entries,
        )
    }

    #[test]
    fn test_json_export() {
        let exporter = JsonExporter::new();
        let session = create_test_session();
        let output = exporter.export(&session);

        assert!(output.contains("\"id\": \"test-session\""));
        assert!(output.contains("\"project\": \"/test/project\""));
        assert!(output.contains("\"role\": \"user\""));
        assert!(output.contains("\"role\": \"assistant\""));
        assert!(output.contains("Hello, Claude!"));
    }

    #[test]
    fn test_json_file_extension() {
        let exporter = JsonExporter::new();
        assert_eq!(exporter.file_extension(), "json");
    }

    #[test]
    fn test_json_compact() {
        let exporter = JsonExporter::new().pretty(false);
        let session = create_test_session();
        let output = exporter.export(&session);

        // 整形されていないので改行が少ない
        assert!(!output.contains("  "));
    }
}
