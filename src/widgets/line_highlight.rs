use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

pub struct LineHighlight {
    row: u16,
    style: Style,
}

impl LineHighlight {
    pub fn new(row: u16, style: Style) -> Self {
        Self { row, style }
    }
}

impl Widget for LineHighlight {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || self.row >= area.height {
            return;
        }
        let line_area = Rect {
            x: area.x,
            y: area.y + self.row,
            width: area.width,
            height: 1,
        };
        buf.set_style(line_area, self.style);
    }
}
