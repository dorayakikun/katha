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
    pub content: String,
}

/// history.jsonl の各行を表すエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// ユーザー入力の表示テキスト
    pub display: String,

    /// 貼り付けコンテンツ（通常は空）
    #[serde(default)]
    pub pasted_contents: HashMap<String, PastedContent>,

    /// ミリ秒単位 Unix タイムスタンプ
    pub timestamp: i64,

    /// プロジェクトパス
    pub project: String,

    /// セッション ID（UUID）
    pub session_id: String,
}

impl HistoryEntry {
    /// タイムスタンプを DateTime<Utc> に変換
    pub fn datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.timestamp)
            .single()
            .unwrap_or(DateTime::UNIX_EPOCH)
    }

    /// プロジェクト名（パスの最後の部分）
    pub fn project_name(&self) -> &str {
        self.project.rsplit('/').next().unwrap_or(&self.project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_history_entry_datetime() {
        let entry = HistoryEntry {
            display: "/init".to_string(),
            pasted_contents: HashMap::new(),
            timestamp: 1735270066438, // 2024-12-27 頃
            project: "/Users/test/project".to_string(),
            session_id: "test-uuid".to_string(),
        };

        let dt = entry.datetime();
        assert!(dt.year() >= 2024);
    }

    #[test]
    fn test_project_name() {
        let entry = HistoryEntry {
            display: "test".to_string(),
            pasted_contents: HashMap::new(),
            timestamp: 0,
            project: "/Users/test/my-project".to_string(),
            session_id: "test".to_string(),
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
        assert_eq!(entry.display, "/init ");
        assert_eq!(entry.session_id, "098d6872-8810-446c-af1f-64586872aa0e");
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
        assert_eq!(entry.display, "/init ");
        assert_eq!(entry.pasted_contents.len(), 1);

        let pasted = entry.pasted_contents.get("1").unwrap();
        assert_eq!(pasted.id, 1);
        assert_eq!(pasted.content_type, "text");
        assert_eq!(pasted.content, "Sample pasted content");
    }
}
