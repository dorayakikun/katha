use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::message::Message;

/// セッションファイルの各行
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    /// 親メッセージ UUID
    #[serde(default)]
    pub parent_uuid: Option<String>,

    /// サイドチェーンかどうか
    #[serde(default)]
    pub is_sidechain: bool,

    /// ユーザータイプ
    #[serde(default)]
    pub user_type: Option<String>,

    /// 作業ディレクトリ
    #[serde(default)]
    pub cwd: Option<String>,

    /// セッション ID
    #[serde(default)]
    pub session_id: Option<String>,

    /// Claude Code バージョン
    #[serde(default)]
    pub version: Option<String>,

    /// Git ブランチ
    #[serde(default)]
    pub git_branch: Option<String>,

    /// エントリタイプ（"user" | "assistant" | etc）
    #[serde(rename = "type")]
    pub entry_type: String,

    /// メッセージ本体
    #[serde(default)]
    pub message: Option<Message>,

    /// メッセージ UUID
    #[serde(default)]
    pub uuid: Option<String>,

    /// ISO 8601 タイムスタンプ
    #[serde(default)]
    pub timestamp: Option<String>,

    /// API リクエスト ID
    #[serde(default)]
    pub request_id: Option<String>,

    /// メタデータメッセージか
    #[serde(default)]
    pub is_meta: bool,

    /// Agent ID
    #[serde(default)]
    pub agent_id: Option<String>,

    /// セッションスラッグ
    #[serde(default)]
    pub slug: Option<String>,
}

impl SessionEntry {
    /// タイムスタンプを DateTime に変換
    pub fn datetime(&self) -> Option<DateTime<Utc>> {
        self.timestamp.as_ref().and_then(|ts| {
            DateTime::parse_from_rfc3339(ts)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        })
    }

    /// ユーザーメッセージか
    pub fn is_user(&self) -> bool {
        self.entry_type == "user" && !self.is_meta
    }

    /// アシスタントメッセージか
    pub fn is_assistant(&self) -> bool {
        self.entry_type == "assistant"
    }

    /// 表示テキストを取得
    pub fn display_text(&self) -> Option<String> {
        self.message.as_ref().and_then(|m| m.text_content())
    }
}

/// セッション全体
#[derive(Debug, Clone)]
pub struct Session {
    /// セッション ID
    pub id: String,

    /// プロジェクトパス
    pub project: String,

    /// スラッグ
    pub slug: Option<String>,

    /// 全エントリ
    pub entries: Vec<SessionEntry>,

    /// 開始時刻
    pub started_at: Option<DateTime<Utc>>,

    /// 終了時刻
    pub ended_at: Option<DateTime<Utc>>,
}

impl Session {
    /// エントリから構築
    pub fn from_entries(id: String, project: String, entries: Vec<SessionEntry>) -> Self {
        let slug = entries.iter().find_map(|e| e.slug.clone());

        let timestamps: Vec<_> = entries.iter().filter_map(|e| e.datetime()).collect();

        let started_at = timestamps.iter().min().copied();
        let ended_at = timestamps.iter().max().copied();

        Self {
            id,
            project,
            slug,
            entries,
            started_at,
            ended_at,
        }
    }

    /// ユーザーメッセージのみ
    pub fn user_messages(&self) -> impl Iterator<Item = &SessionEntry> {
        self.entries.iter().filter(|e| e.is_user())
    }

    /// アシスタントメッセージのみ
    pub fn assistant_messages(&self) -> impl Iterator<Item = &SessionEntry> {
        self.entries.iter().filter(|e| e.is_assistant())
    }

    /// プロジェクト名
    pub fn project_name(&self) -> &str {
        self.project.rsplit('/').next().unwrap_or(&self.project)
    }

    /// 最初のユーザーメッセージ
    pub fn first_user_message(&self) -> Option<&SessionEntry> {
        self.entries.iter().find(|e| e.is_user())
    }

    /// メッセージ数
    pub fn message_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.is_user() || e.is_assistant())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_session_entry_datetime() {
        let entry = SessionEntry {
            parent_uuid: None,
            is_sidechain: false,
            user_type: None,
            cwd: None,
            session_id: None,
            version: None,
            git_branch: None,
            entry_type: "user".to_string(),
            message: None,
            uuid: None,
            timestamp: Some("2025-12-27T03:47:49.992Z".to_string()),
            request_id: None,
            is_meta: false,
            agent_id: None,
            slug: None,
        };

        let dt = entry.datetime().unwrap();
        assert_eq!(dt.year(), 2025);
    }

    #[test]
    fn test_session_entry_is_user() {
        let user = SessionEntry {
            entry_type: "user".to_string(),
            is_meta: false,
            ..Default::default()
        };
        assert!(user.is_user());

        let meta = SessionEntry {
            entry_type: "user".to_string(),
            is_meta: true,
            ..Default::default()
        };
        assert!(!meta.is_user());
    }

    #[test]
    fn test_session_from_entries() {
        let entries = vec![
            SessionEntry {
                entry_type: "user".to_string(),
                timestamp: Some("2025-01-01T00:00:00Z".to_string()),
                slug: Some("test-slug".to_string()),
                ..Default::default()
            },
            SessionEntry {
                entry_type: "assistant".to_string(),
                timestamp: Some("2025-01-01T00:01:00Z".to_string()),
                ..Default::default()
            },
        ];

        let session =
            Session::from_entries("test-id".to_string(), "/test/project".to_string(), entries);

        assert_eq!(session.id, "test-id");
        assert_eq!(session.slug, Some("test-slug".to_string()));
        assert!(session.started_at.is_some());
        assert_eq!(session.message_count(), 2);
    }
}
