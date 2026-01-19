use crate::search::{FilterCriteria, SearchQuery};
use crate::tea::SessionListItem;

/// 検索エンジン
pub struct SearchEngine;

impl SearchEngine {
    /// 検索クエリでフィルタリング（project_name, latest_user_message を検索）
    pub fn search(sessions: &[SessionListItem], query: &SearchQuery) -> Vec<usize> {
        if query.is_empty() {
            return (0..sessions.len()).collect();
        }

        sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| {
                query.matches(&session.project_name)
                    || query.matches(&session.latest_user_message)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// フィルタ条件でフィルタリング
    pub fn filter(sessions: &[SessionListItem], criteria: &FilterCriteria) -> Vec<usize> {
        if !criteria.is_set() {
            return (0..sessions.len()).collect();
        }

        sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| Self::matches_criteria(session, criteria))
            .map(|(i, _)| i)
            .collect()
    }

    /// 検索 + フィルタの組み合わせ（AND条件）
    pub fn search_and_filter(
        sessions: &[SessionListItem],
        query: &SearchQuery,
        criteria: &FilterCriteria,
    ) -> Vec<usize> {
        sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| {
                // 検索クエリにマッチ
                let matches_query = query.is_empty()
                    || query.matches(&session.project_name)
                    || query.matches(&session.latest_user_message);

                // フィルタ条件にマッチ
                let matches_filter =
                    !criteria.is_set() || Self::matches_criteria(session, criteria);

                matches_query && matches_filter
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// セッションがフィルタ条件にマッチするか
    fn matches_criteria(session: &SessionListItem, criteria: &FilterCriteria) -> bool {
        // 日付範囲チェック
        if criteria.date_range.is_set() && !criteria.date_range.contains(session.datetime) {
            return false;
        }

        // プロジェクトフィルタチェック
        if let Some(ref project_filter) = criteria.project_filter
            && !project_filter.is_empty()
            && !session
                .project_name
                .to_lowercase()
                .contains(&project_filter.to_lowercase())
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::DateRange;
    use chrono::{TimeZone, Utc};

    fn create_test_sessions() -> Vec<SessionListItem> {
        vec![
            SessionListItem {
                session_id: "1".to_string(),
                project_name: "my-project".to_string(),
                project_path: "/path/to/my-project".to_string(),
                latest_user_message: "Hello world".to_string(),
                formatted_time: "2025-01-15 10:00".to_string(),
                datetime: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
            },
            SessionListItem {
                session_id: "2".to_string(),
                project_name: "another-app".to_string(),
                project_path: "/path/to/another-app".to_string(),
                latest_user_message: "Fix bug in login".to_string(),
                formatted_time: "2025-01-10 14:30".to_string(),
                datetime: Utc.with_ymd_and_hms(2025, 1, 10, 14, 30, 0).unwrap(),
            },
            SessionListItem {
                session_id: "3".to_string(),
                project_name: "my-project".to_string(),
                project_path: "/path/to/my-project".to_string(),
                latest_user_message: "Add new feature".to_string(),
                formatted_time: "2025-01-05 09:00".to_string(),
                datetime: Utc.with_ymd_and_hms(2025, 1, 5, 9, 0, 0).unwrap(),
            },
        ]
    }

    #[test]
    fn test_search_empty_query() {
        let sessions = create_test_sessions();
        let query = SearchQuery::default();

        let result = SearchEngine::search(&sessions, &query);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_search_by_project_name() {
        let sessions = create_test_sessions();
        let query = SearchQuery {
            text: "my-project".to_string(),
            case_sensitive: false,
        };

        let result = SearchEngine::search(&sessions, &query);
        assert_eq!(result, vec![0, 2]); // Sessions 0 and 2 have "my-project"
    }

    #[test]
    fn test_search_by_latest_message() {
        let sessions = create_test_sessions();
        let query = SearchQuery {
            text: "bug".to_string(),
            case_sensitive: false,
        };

        let result = SearchEngine::search(&sessions, &query);
        assert_eq!(result, vec![1]); // Session 1 has "Fix bug"
    }

    #[test]
    fn test_search_case_insensitive() {
        let sessions = create_test_sessions();
        let query = SearchQuery {
            text: "HELLO".to_string(),
            case_sensitive: false,
        };

        let result = SearchEngine::search(&sessions, &query);
        assert_eq!(result, vec![0]); // Session 0 has "Hello world"
    }

    #[test]
    fn test_search_no_match() {
        let sessions = create_test_sessions();
        let query = SearchQuery {
            text: "xyz".to_string(),
            case_sensitive: false,
        };

        let result = SearchEngine::search(&sessions, &query);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_empty_criteria() {
        let sessions = create_test_sessions();
        let criteria = FilterCriteria::default();

        let result = SearchEngine::filter(&sessions, &criteria);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_filter_by_date_range() {
        let sessions = create_test_sessions();
        let criteria = FilterCriteria {
            date_range: DateRange {
                from: Some(Utc.with_ymd_and_hms(2025, 1, 10, 0, 0, 0).unwrap()),
                to: Some(Utc.with_ymd_and_hms(2025, 1, 20, 0, 0, 0).unwrap()),
            },
            project_filter: None,
        };

        let result = SearchEngine::filter(&sessions, &criteria);
        assert_eq!(result, vec![0, 1]); // Sessions 0 and 1 are within range
    }

    #[test]
    fn test_filter_by_project() {
        let sessions = create_test_sessions();
        let criteria = FilterCriteria {
            date_range: DateRange::default(),
            project_filter: Some("another".to_string()),
        };

        let result = SearchEngine::filter(&sessions, &criteria);
        assert_eq!(result, vec![1]); // Only session 1 matches "another"
    }

    #[test]
    fn test_search_and_filter_combined() {
        let sessions = create_test_sessions();
        let query = SearchQuery {
            text: "my-project".to_string(),
            case_sensitive: false,
        };
        let criteria = FilterCriteria {
            date_range: DateRange {
                from: Some(Utc.with_ymd_and_hms(2025, 1, 10, 0, 0, 0).unwrap()),
                to: None,
            },
            project_filter: None,
        };

        let result = SearchEngine::search_and_filter(&sessions, &query, &criteria);
        assert_eq!(result, vec![0]); // Only session 0 matches both query and date filter
    }

    #[test]
    fn test_filter_by_project_case_insensitive() {
        let sessions = create_test_sessions();
        let criteria = FilterCriteria {
            date_range: DateRange::default(),
            project_filter: Some("ANOTHER".to_string()),
        };

        let result = SearchEngine::filter(&sessions, &criteria);
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_filter_empty_project_filter() {
        let sessions = create_test_sessions();
        let criteria = FilterCriteria {
            date_range: DateRange::default(),
            project_filter: Some(String::new()), // Empty string should not filter
        };

        let result = SearchEngine::filter(&sessions, &criteria);
        assert_eq!(result, vec![0, 1, 2]);
    }
}
