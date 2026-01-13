use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::color::ColorScale;

/// Minimum terminal width to show legend
pub const MIN_WIDTH_FOR_LEGEND: u16 = 100;
/// Width of the legend panel
pub const LEGEND_WIDTH: u16 = 16;

/// Legend widget showing color scale
pub struct Legend<'a> {
    color_scale: &'a ColorScale,
}

impl<'a> Legend<'a> {
    pub fn new(color_scale: &'a ColorScale) -> Self {
        Self { color_scale }
    }
}

impl Widget for Legend<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let entries = self.color_scale.legend_entries();

        let lines: Vec<Line> = entries
            .iter()
            .map(|(color, label)| {
                Line::from(vec![
                    Span::styled("█ ", Style::default().fg(*color)),
                    Span::styled(label, Style::default().fg(Color::Gray)),
                ])
            })
            .collect();

        let block = Block::default()
            .title(" Legend ")
            .title_style(Style::default().fg(Color::DarkGray))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let paragraph = Paragraph::new(lines).block(block);
        paragraph.render(area, buf);
    }
}
