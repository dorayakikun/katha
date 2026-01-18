/// 検索クエリ
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// 検索テキスト
    pub text: String,
    /// 大文字小文字を区別するか
    pub case_sensitive: bool,
}

impl SearchQuery {
    /// クエリが空かどうか
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 対象テキストがクエリにマッチするか
    /// case_sensitive が false の場合は大文字小文字を無視
    pub fn matches(&self, target: &str) -> bool {
        if self.text.is_empty() {
            return true;
        }

        if self.case_sensitive {
            target.contains(&self.text)
        } else {
            target.to_lowercase().contains(&self.text.to_lowercase())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        let query = SearchQuery::default();
        assert!(query.is_empty());

        let query = SearchQuery {
            text: "test".to_string(),
            case_sensitive: false,
        };
        assert!(!query.is_empty());
    }

    #[test]
    fn test_matches_empty_query() {
        let query = SearchQuery::default();
        assert!(query.matches("anything"));
        assert!(query.matches(""));
    }

    #[test]
    fn test_matches_case_insensitive() {
        let query = SearchQuery {
            text: "test".to_string(),
            case_sensitive: false,
        };

        assert!(query.matches("test"));
        assert!(query.matches("TEST"));
        assert!(query.matches("Test"));
        assert!(query.matches("this is a test"));
        assert!(query.matches("THIS IS A TEST"));
        assert!(!query.matches("no match"));
    }

    #[test]
    fn test_matches_case_sensitive() {
        let query = SearchQuery {
            text: "Test".to_string(),
            case_sensitive: true,
        };

        assert!(query.matches("Test"));
        assert!(query.matches("This is a Test"));
        assert!(!query.matches("test"));
        assert!(!query.matches("TEST"));
    }

    #[test]
    fn test_matches_partial() {
        let query = SearchQuery {
            text: "hello".to_string(),
            case_sensitive: false,
        };

        assert!(query.matches("hello world"));
        assert!(query.matches("say hello"));
        assert!(query.matches("helloooo"));
    }
}
