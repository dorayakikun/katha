use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 貼り付けコンテンツの詳細
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastedContent {
    /// コンテンツID
    pub id: u64,

    /// コンテンツタイプ（"text" など）
    #[serde(rename = "type")]
    pub content_type: String,

    /// コンテンツ本体
    #[serde(default)]
    pub content: String,
}

/// history.jsonl の各行を表すエントリ
/// 緩いスキーマ設計：すべてのフィールドを Option にしてパースエラーを防ぐ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// ユーザー入力の表示テキスト
    #[serde(default)]
    pub display: Option<String>,

    /// 貼り付けコンテンツ（通常は空）
    #[serde(default)]
    pub pasted_contents: HashMap<String, PastedContent>,

    /// ミリ秒単位 Unix タイムスタンプ
    #[serde(default)]
    pub timestamp: Option<i64>,

    /// プロジェクトパス
    #[serde(default)]
    pub project: Option<String>,

    /// セッション ID（UUID）
    #[serde(default)]
    pub session_id: Option<String>,
}

impl HistoryEntry {
    /// エントリが有効かどうか（最低限必要なフィールドが存在するか）
    pub fn is_valid(&self) -> bool {
        self.session_id.is_some()
    }

    /// タイムスタンプを DateTime<Utc> に変換
    pub fn datetime(&self) -> DateTime<Utc> {
        self.timestamp
            .and_then(|ts| Utc.timestamp_millis_opt(ts).single())
            .unwrap_or(DateTime::UNIX_EPOCH)
    }

    /// プロジェクトパスを取得（デフォルト: "unknown"）
    pub fn project(&self) -> &str {
        self.project.as_deref().unwrap_or("unknown")
    }

    /// プロジェクト名（パスの最後の部分）
    pub fn project_name(&self) -> &str {
        let project = self.project();
        project.rsplit('/').next().unwrap_or(project)
    }

    /// 表示テキストを取得（デフォルト: ""）
    pub fn display(&self) -> &str {
        self.display.as_deref().unwrap_or("")
    }

    /// セッション ID を取得
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_history_entry_datetime() {
        let entry = HistoryEntry {
            display: Some("/init".to_string()),
            pasted_contents: HashMap::new(),
            timestamp: Some(1735270066438), // 2024-12-27 頃
            project: Some("/Users/test/project".to_string()),
            session_id: Some("test-uuid".to_string()),
        };

        let dt = entry.datetime();
        assert!(dt.year() >= 2024);
    }

    #[test]
    fn test_project_name() {
        let entry = HistoryEntry {
            display: Some("test".to_string()),
            pasted_contents: HashMap::new(),
            timestamp: Some(0),
            project: Some("/Users/test/my-project".to_string()),
            session_id: Some("test".to_string()),
        };

        assert_eq!(entry.project_name(), "my-project");
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "display": "/init ",
            "pastedContents": {},
            "timestamp": 1766807266438,
            "project": "/Users/test/project",
            "sessionId": "098d6872-8810-446c-af1f-64586872aa0e"
        }"#;

        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.display(), "/init ");
        assert_eq!(entry.session_id(), Some("098d6872-8810-446c-af1f-64586872aa0e"));
        assert!(entry.is_valid());
    }

    #[test]
    fn test_deserialize_with_pasted_contents() {
        let json = r#"{
            "display": "/init ",
            "pastedContents": {
                "1": {"id": 1, "type": "text", "content": "Sample pasted content"}
            },
            "timestamp": 1766807266438,
            "project": "/Users/test/project",
            "sessionId": "098d6872-8810-446c-af1f-64586872aa0e"
        }"#;

        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.display(), "/init ");
        assert_eq!(entry.pasted_contents.len(), 1);

        let pasted = entry.pasted_contents.get("1").unwrap();
        assert_eq!(pasted.id, 1);
        assert_eq!(pasted.content_type, "text");
        assert_eq!(pasted.content, "Sample pasted content");
    }

    #[test]
    fn test_deserialize_missing_session_id() {
        // sessionId が欠落しているケース（緩いスキーマでパースは成功するが is_valid() は false）
        let json = r#"{
            "display": "/init ",
            "pastedContents": {},
            "timestamp": 1766807266438,
            "project": "/Users/test/project"
        }"#;

        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert!(!entry.is_valid());
        assert_eq!(entry.session_id(), None);
    }

    #[test]
    fn test_deserialize_minimal() {
        // 最小限のフィールドのみ
        let json = r#"{}"#;

        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert!(!entry.is_valid());
        assert_eq!(entry.display(), "");
        assert_eq!(entry.project(), "unknown");
    }

    #[test]
    fn test_is_valid() {
        let valid = HistoryEntry {
            session_id: Some("test".to_string()),
            ..Default::default()
        };
        assert!(valid.is_valid());

        let invalid = HistoryEntry::default();
        assert!(!invalid.is_valid());
    }
}
