use chrono::{DateTime, Duration, Utc};

/// 日付範囲
#[derive(Debug, Clone, Default)]
pub struct DateRange {
    /// 開始日時（含む）
    pub from: Option<DateTime<Utc>>,
    /// 終了日時（含む）
    pub to: Option<DateTime<Utc>>,
}

impl DateRange {
    /// 今日（UTC ベース）
    pub fn today() -> Self {
        let now = Utc::now();
        let start_of_today = now.date_naive().and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());

        Self {
            from: start_of_today,
            to: Some(now),
        }
    }

    /// 過去1週間
    pub fn last_week() -> Self {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);

        Self {
            from: Some(week_ago),
            to: Some(now),
        }
    }

    /// 過去1ヶ月
    pub fn last_month() -> Self {
        let now = Utc::now();
        let month_ago = now - Duration::days(30);

        Self {
            from: Some(month_ago),
            to: Some(now),
        }
    }

    /// 範囲が設定されているか
    pub fn is_set(&self) -> bool {
        self.from.is_some() || self.to.is_some()
    }

    /// 日時が範囲内かどうか
    pub fn contains(&self, dt: DateTime<Utc>) -> bool {
        if let Some(from) = self.from
            && dt < from
        {
            return false;
        }
        if let Some(to) = self.to
            && dt > to
        {
            return false;
        }
        true
    }
}

/// フィルタ条件
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    /// 日付範囲
    pub date_range: DateRange,
    /// プロジェクトフィルタ（部分一致）
    pub project_filter: Option<String>,
}

impl FilterCriteria {
    /// フィルタが設定されているか
    pub fn is_set(&self) -> bool {
        self.date_range.is_set() || self.project_filter.is_some()
    }

    /// フィルタをクリア
    pub fn clear(&mut self) {
        self.date_range = DateRange::default();
        self.project_filter = None;
    }
}

/// フィルタパネルのフィールド
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterField {
    /// 日付範囲
    #[default]
    DateRange,
    /// プロジェクト
    Project,
}

impl FilterField {
    /// 次のフィールドに移動
    pub fn next(self) -> Self {
        match self {
            FilterField::DateRange => FilterField::Project,
            FilterField::Project => FilterField::DateRange,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_date_range_default() {
        let range = DateRange::default();
        assert!(range.from.is_none());
        assert!(range.to.is_none());
        assert!(!range.is_set());
    }

    #[test]
    fn test_date_range_today() {
        let range = DateRange::today();
        assert!(range.is_set());
        assert!(range.from.is_some());
        assert!(range.to.is_some());
    }

    #[test]
    fn test_date_range_last_week() {
        let range = DateRange::last_week();
        assert!(range.is_set());

        let from = range.from.unwrap();
        let to = range.to.unwrap();
        let diff = to - from;
        assert_eq!(diff.num_days(), 7);
    }

    #[test]
    fn test_date_range_last_month() {
        let range = DateRange::last_month();
        assert!(range.is_set());

        let from = range.from.unwrap();
        let to = range.to.unwrap();
        let diff = to - from;
        assert_eq!(diff.num_days(), 30);
    }

    #[test]
    fn test_date_range_contains() {
        let range = DateRange {
            from: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
            to: Some(Utc.with_ymd_and_hms(2025, 1, 31, 23, 59, 59).unwrap()),
        };

        // 範囲内
        let mid = Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        assert!(range.contains(mid));

        // 範囲前
        let before = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        assert!(!range.contains(before));

        // 範囲後
        let after = Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();
        assert!(!range.contains(after));
    }

    #[test]
    fn test_date_range_contains_no_limit() {
        let range = DateRange::default();
        let any_date = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        assert!(range.contains(any_date));
    }

    #[test]
    fn test_filter_criteria_default() {
        let criteria = FilterCriteria::default();
        assert!(!criteria.is_set());
    }

    #[test]
    fn test_filter_criteria_with_date_range() {
        let criteria = FilterCriteria {
            date_range: DateRange::today(),
            project_filter: None,
        };
        assert!(criteria.is_set());
    }

    #[test]
    fn test_filter_criteria_with_project() {
        let criteria = FilterCriteria {
            date_range: DateRange::default(),
            project_filter: Some("my-project".to_string()),
        };
        assert!(criteria.is_set());
    }

    #[test]
    fn test_filter_criteria_clear() {
        let mut criteria = FilterCriteria {
            date_range: DateRange::today(),
            project_filter: Some("test".to_string()),
        };

        criteria.clear();
        assert!(!criteria.is_set());
        assert!(criteria.date_range.from.is_none());
        assert!(criteria.project_filter.is_none());
    }

    #[test]
    fn test_filter_field_next() {
        assert_eq!(FilterField::DateRange.next(), FilterField::Project);
        assert_eq!(FilterField::Project.next(), FilterField::DateRange);
    }
}
