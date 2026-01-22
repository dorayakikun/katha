use crate::domain::Session;

use super::Exporter;

/// Markdown エクスポータ
pub struct MarkdownExporter;

impl MarkdownExporter {
    /// 新規作成
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for MarkdownExporter {
    fn export(&self, session: &Session) -> String {
        let mut output = String::new();

        // タイトル
        output.push_str(&format!("# Session: {}\n\n", session.project_name()));

        // メタデータ
        output.push_str(&format!("- **Project**: {}\n", session.project));

        if let Some(started) = session.started_at {
            let start_str = started.format("%Y-%m-%d %H:%M").to_string();
            if let Some(ended) = session.ended_at {
                let end_str = ended.format("%H:%M").to_string();
                output.push_str(&format!("- **Date**: {} - {}\n", start_str, end_str));
            } else {
                output.push_str(&format!("- **Date**: {}\n", start_str));
            }
        }

        output.push_str(&format!("- **Messages**: {}\n", session.message_count()));

        if let Some(slug) = &session.slug {
            output.push_str(&format!("- **Slug**: {}\n", slug));
        }

        output.push_str("\n---\n\n");

        // メッセージ（テキストがある場合のみ出力）
        for entry in &session.entries {
            if entry.is_user() {
                if let Some(text) = entry.display_text() {
                    output.push_str("## User\n\n");
                    output.push_str(&text);
                    output.push_str("\n\n---\n\n");
                }
            } else if entry.is_assistant() {
                if let Some(text) = entry.display_text() {
                    output.push_str("## Assistant\n\n");
                    output.push_str(&text);
                    output.push_str("\n\n---\n\n");
                }
            }
        }

        output
    }

    fn file_extension(&self) -> &'static str {
        "md"
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
                entry_type: Some("user".to_string()),
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
                entry_type: Some("assistant".to_string()),
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
    fn test_markdown_export() {
        let exporter = MarkdownExporter::new();
        let session = create_test_session();
        let output = exporter.export(&session);

        assert!(output.contains("# Session: project"));
        assert!(output.contains("**Project**: /test/project"));
        assert!(output.contains("## User"));
        assert!(output.contains("Hello, Claude!"));
        assert!(output.contains("## Assistant"));
        assert!(output.contains("Hello! How can I help you?"));
    }

    #[test]
    fn test_markdown_file_extension() {
        let exporter = MarkdownExporter::new();
        assert_eq!(exporter.file_extension(), "md");
    }

    #[test]
    fn test_markdown_export_skips_empty_messages() {
        use crate::domain::message::ContentBlock;

        // thinking のみのメッセージを含むセッション
        let entries = vec![
            SessionEntry {
                entry_type: Some("user".to_string()),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                message: Some(Message {
                    role: "user".to_string(),
                    content: MessageContent::Text("Hello!".to_string()),
                    model: None,
                    id: None,
                    stop_reason: None,
                    usage: None,
                }),
                ..Default::default()
            },
            SessionEntry {
                entry_type: Some("assistant".to_string()),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                message: Some(Message {
                    role: "assistant".to_string(),
                    content: MessageContent::Blocks(vec![ContentBlock::Thinking {
                        thinking: "...".to_string(),
                    }]),
                    model: None,
                    id: None,
                    stop_reason: None,
                    usage: None,
                }),
                ..Default::default()
            },
        ];

        let session = Session::from_entries(
            "test-session".to_string(),
            "/test/project".to_string(),
            entries,
        );

        let exporter = MarkdownExporter::new();
        let output = exporter.export(&session);

        // User セクションは存在する
        assert!(output.contains("## User"));
        assert!(output.contains("Hello!"));

        // thinking のみのアシスタントメッセージは出力されない
        // "## Assistant" が1回も出現しないことを確認
        assert!(!output.contains("## Assistant"));
    }
}
