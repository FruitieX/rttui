use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use super::app::SettingsField;
use crate::color::ColorScheme;

/// Settings menu widget
pub struct SettingsMenu {
    pub selected_field: SettingsField,
    pub target: String,
    pub interval: u64,
    pub scale: u64,
    pub scale_min: u64,
    pub scale_max: u64,
    pub auto_scale: bool,
    pub colors: ColorScheme,
    pub show_cursor: bool,
    pub sparkline: bool,
    pub defer_row: bool,
    pub buffer_mb: u64,
    pub input_active: bool,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub input_selected: bool,
}

impl SettingsMenu {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        selected_field: SettingsField,
        target: String,
        interval: u64,
        scale: u64,
        scale_min: u64,
        scale_max: u64,
        auto_scale: bool,
        colors: ColorScheme,
        show_cursor: bool,
        sparkline: bool,
        defer_row: bool,
        buffer_mb: u64,
        input_active: bool,
        input_buffer: String,
        input_cursor: usize,
        input_selected: bool,
    ) -> Self {
        Self {
            selected_field,
            target,
            interval,
            scale,
            scale_min,
            scale_max,
            auto_scale,
            colors,
            show_cursor,
            sparkline,
            defer_row,
            buffer_mb,
            input_active,
            input_buffer,
            input_cursor,
            input_selected,
        }
    }
}

impl Widget for SettingsMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate centered position for the settings box (wider now)
        let width = 65u16.min(area.width.saturating_sub(4));
        let field_count: u16 = if self.auto_scale { 10 } else { 9 };
        let height = (field_count * 2 + 7).min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        let menu_area = Rect::new(x, y, width, height);

        // Clear the background
        Clear.render(menu_area, buf);

        let block = Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));

        let inner_area = block.inner(menu_area);
        block.render(menu_area, buf);

        // Build settings lines
        let normal_style = Style::default().fg(Color::White);
        let selected_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let label_style = Style::default().fg(Color::Gray);
        let value_style = Style::default().fg(Color::Cyan);
        let hint_style = Style::default().fg(Color::DarkGray);
        let input_style = Style::default().fg(Color::White).bg(Color::Rgb(60, 60, 80));
        let selected_text_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(150, 180, 255));
        let button_style = Style::default().fg(Color::White).bg(Color::Rgb(60, 60, 80));
        let button_selected_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(100, 200, 100));
        let cancel_selected_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(200, 100, 100));

        // Helper to show value or input buffer with cursor
        let show_value = |field: SettingsField, value: &str| -> Vec<Span> {
            if self.input_active && self.selected_field == field {
                if self.input_selected {
                    // Show entire text as selected
                    vec![Span::styled(self.input_buffer.clone(), selected_text_style)]
                } else {
                    // Show with cursor at position
                    let before: String =
                        self.input_buffer.chars().take(self.input_cursor).collect();
                    let after: String = self.input_buffer.chars().skip(self.input_cursor).collect();
                    vec![
                        Span::styled(before, input_style),
                        Span::styled("▏", Style::default().fg(Color::White)),
                        Span::styled(after, input_style),
                    ]
                }
            } else {
                vec![Span::styled(
                    value.to_string(),
                    if self.selected_field == field {
                        selected_style
                    } else {
                        value_style
                    },
                )]
            }
        };

        let target_spans = show_value(SettingsField::Target, &self.target);
        let interval_spans = show_value(SettingsField::Interval, &format!("{}", self.interval));
        let scale_spans = show_value(SettingsField::Scale, &format!("{}", self.scale));
        let scale_min_spans = show_value(SettingsField::ScaleMin, &format!("{}", self.scale_min));
        let scale_max_spans = show_value(SettingsField::ScaleMax, &format!("{}", self.scale_max));
        let buffer_spans = show_value(SettingsField::BufferSize, &format!("{}", self.buffer_mb));

        // Build target line
        let mut target_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::Target {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::Target {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Target:       ", label_style),
        ];
        target_line.extend(target_spans);

        // Build interval line
        let mut interval_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::Interval {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::Interval {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Interval:     ", label_style),
        ];
        interval_line.extend(interval_spans);
        interval_line.push(Span::styled(" ms", label_style));

        // Build scale line
        let mut scale_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::Scale {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::Scale {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Scale:        ", label_style),
        ];
        scale_line.extend(scale_spans);
        scale_line.push(Span::styled(
            " ms (max RTT for color gradient)",
            label_style,
        ));

        let mut scale_min_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::ScaleMin {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::ScaleMin {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Scale Min:    ", label_style),
        ];
        scale_min_line.extend(scale_min_spans);
        scale_min_line.push(Span::styled(" ms (auto-scale lower bound)", label_style));

        let mut scale_max_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::ScaleMax {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::ScaleMax {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Scale Max:    ", label_style),
        ];
        scale_max_line.extend(scale_max_spans);
        scale_max_line.push(Span::styled(" ms (auto-scale upper bound)", label_style));

        // Build buffer size line
        let mut buffer_line = vec![
            Span::styled(
                if self.selected_field == SettingsField::BufferSize {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::BufferSize {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Buffer Size:  ", label_style),
        ];
        buffer_line.extend(buffer_spans);
        buffer_line.push(Span::styled(" MB (history scrollback)", label_style));

        let mut lines = vec![Line::from("")];
        lines.push(Line::from(target_line));
        lines.push(Line::from(""));
        lines.push(Line::from(interval_line));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                if self.selected_field == SettingsField::AutoScale {
                    "► "
                } else {
                    "  "
                },
                if self.selected_field == SettingsField::AutoScale {
                    selected_style
                } else {
                    normal_style
                },
            ),
            Span::styled("Auto Scale:   ", label_style),
            Span::styled(
                if self.auto_scale { "Yes" } else { "No" },
                if self.selected_field == SettingsField::AutoScale {
                    selected_style
                } else {
                    value_style
                },
            ),
        ]));
        lines.push(Line::from(""));

        if self.auto_scale {
            lines.push(Line::from(scale_min_line));
            lines.push(Line::from(""));
            lines.push(Line::from(scale_max_line));
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(scale_line));
            lines.push(Line::from(""));
        }

        lines.extend(vec![
            Line::from(vec![
                Span::styled(
                    if self.selected_field == SettingsField::ColorScheme {
                        "► "
                    } else {
                        "  "
                    },
                    if self.selected_field == SettingsField::ColorScheme {
                        selected_style
                    } else {
                        normal_style
                    },
                ),
                Span::styled("Color Scheme: ", label_style),
                Span::styled(
                    format!("{}", self.colors),
                    if self.selected_field == SettingsField::ColorScheme {
                        selected_style
                    } else {
                        value_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if self.selected_field == SettingsField::ShowCursor {
                        "► "
                    } else {
                        "  "
                    },
                    if self.selected_field == SettingsField::ShowCursor {
                        selected_style
                    } else {
                        normal_style
                    },
                ),
                Span::styled("Show Cursor:  ", label_style),
                Span::styled(
                    if self.show_cursor { "Yes" } else { "No" },
                    if self.selected_field == SettingsField::ShowCursor {
                        selected_style
                    } else {
                        value_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if self.selected_field == SettingsField::Sparkline {
                        "► "
                    } else {
                        "  "
                    },
                    if self.selected_field == SettingsField::Sparkline {
                        selected_style
                    } else {
                        normal_style
                    },
                ),
                Span::styled("Sparkline:    ", label_style),
                Span::styled(
                    if self.sparkline { "Yes" } else { "No" },
                    if self.selected_field == SettingsField::Sparkline {
                        selected_style
                    } else {
                        value_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if self.selected_field == SettingsField::DeferRow {
                        "► "
                    } else {
                        "  "
                    },
                    if self.selected_field == SettingsField::DeferRow {
                        selected_style
                    } else {
                        normal_style
                    },
                ),
                Span::styled("Defer Row:    ", label_style),
                Span::styled(
                    if self.defer_row { "Yes" } else { "No" },
                    if self.selected_field == SettingsField::DeferRow {
                        selected_style
                    } else {
                        value_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(buffer_line),
            Line::from(""),
            Line::from(vec![
                Span::raw("                    "),
                Span::styled(
                    " Confirm ",
                    if self.selected_field == SettingsField::Confirm {
                        button_selected_style
                    } else {
                        button_style
                    },
                ),
                Span::raw("    "),
                Span::styled(
                    " Cancel ",
                    if self.selected_field == SettingsField::Cancel {
                        cancel_selected_style
                    } else {
                        button_style
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  ↑/↓ navigate │ ←/→ adjust │ type to edit",
                hint_style,
            )]),
            Line::from(vec![Span::styled(
                "  Enter select │ Esc cancel",
                hint_style,
            )]),
        ]);

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
        paragraph.render(inner_area, buf);
    }
}
