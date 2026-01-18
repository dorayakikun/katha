use serde::{Deserialize, Serialize};
use serde_json::Value;

/// メッセージ本体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// ロール（"user" | "assistant"）
    pub role: String,

    /// コンテンツ（文字列または ContentBlock 配列）
    pub content: MessageContent,

    /// モデル名（assistant のみ）
    #[serde(default)]
    pub model: Option<String>,

    /// メッセージ ID（assistant のみ）
    #[serde(default)]
    pub id: Option<String>,

    /// 停止理由
    #[serde(default)]
    pub stop_reason: Option<String>,

    /// トークン使用量
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// メッセージコンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// コンテンツブロック
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        #[serde(default)]
        is_error: bool,
    },
    Image {
        source: ImageSource,
    },
    Thinking {
        thinking: String,
    },
}

/// ツール結果コンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// 画像ソース
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: Option<String>,
    pub data: Option<String>,
}

/// トークン使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

impl Message {
    /// 最初のテキストコンテンツを取得
    pub fn text_content(&self) -> Option<String> {
        match &self.content {
            MessageContent::Text(s) => Some(Self::clean_text(s)),
            MessageContent::Blocks(blocks) => blocks.iter().find_map(|b| {
                if let ContentBlock::Text { text } = b {
                    Some(Self::clean_text(text))
                } else {
                    None
                }
            }),
        }
    }

    /// 全テキストを結合して取得
    pub fn all_text_content(&self) -> String {
        match &self.content {
            MessageContent::Text(s) => Self::clean_text(s),
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .filter_map(|b| {
                    if let ContentBlock::Text { text } = b {
                        Some(Self::clean_text(text))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// ツール呼び出しを取得
    pub fn tool_uses(&self) -> Vec<(&str, &str, &Value)> {
        match &self.content {
            MessageContent::Text(_) => vec![],
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .filter_map(|b| {
                    if let ContentBlock::ToolUse { id, name, input } = b {
                        Some((id.as_str(), name.as_str(), input))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    /// XMLタグを除去
    fn clean_text(s: &str) -> String {
        let mut result = s.to_string();

        // <command-name>...</command-name> を除去
        while let Some(start) = result.find("<command-name>") {
            if let Some(end) = result.find("</command-name>") {
                result = format!("{}{}", &result[..start], &result[end + 15..]);
            } else {
                break;
            }
        }

        // <command-message>...</command-message> を除去
        while let Some(start) = result.find("<command-message>") {
            if let Some(end) = result.find("</command-message>") {
                result = format!("{}{}", &result[..start], &result[end + 18..]);
            } else {
                break;
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_text_content() {
        let msg = Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello, world!".to_string()),
            model: None,
            id: None,
            stop_reason: None,
            usage: None,
        };
        assert_eq!(msg.text_content(), Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_message_blocks_content() {
        let msg = Message {
            role: "assistant".to_string(),
            content: MessageContent::Blocks(vec![
                ContentBlock::Text {
                    text: "First".to_string(),
                },
                ContentBlock::Text {
                    text: "Second".to_string(),
                },
            ]),
            model: None,
            id: None,
            stop_reason: None,
            usage: None,
        };
        assert_eq!(msg.text_content(), Some("First".to_string()));
        assert_eq!(msg.all_text_content(), "First\nSecond");
    }

    #[test]
    fn test_clean_text_removes_command_tags() {
        let input = "<command-message>init</command-message>\n<command-name>/init</command-name>";
        assert_eq!(Message::clean_text(input), "");
    }

    #[test]
    fn test_deserialize_text_content() {
        let json = r#"{"role":"user","content":"Hello"}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.text_content(), Some("Hello".to_string()));
    }

    #[test]
    fn test_deserialize_blocks_content() {
        let json = r#"{"role":"assistant","content":[{"type":"text","text":"Response"}]}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.text_content(), Some("Response".to_string()));
    }
}
