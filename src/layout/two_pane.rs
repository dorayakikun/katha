use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 2ペインレイアウト
pub struct TwoPane {
    /// 左右の比率（左:右）
    left_ratio: u16,
    right_ratio: u16,
}

impl Default for TwoPane {
    fn default() -> Self {
        Self::new(60, 40)
    }
}

impl TwoPane {
    /// 比率を指定して作成
    pub fn new(left: u16, right: u16) -> Self {
        Self {
            left_ratio: left,
            right_ratio: right,
        }
    }

    /// 領域を左右に分割
    pub fn split(&self, area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_ratio),
                Constraint::Percentage(self.right_ratio),
            ])
            .split(area);

        (chunks[0], chunks[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_pane_default() {
        let pane = TwoPane::default();
        assert_eq!(pane.left_ratio, 60);
        assert_eq!(pane.right_ratio, 40);
    }

    #[test]
    fn test_two_pane_split() {
        let pane = TwoPane::new(50, 50);
        let area = Rect::new(0, 0, 100, 50);
        let (left, right) = pane.split(area);

        assert_eq!(left.width, 50);
        assert_eq!(right.width, 50);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 50);
    }
}
