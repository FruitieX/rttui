use crate::color::{ColorScale, ColorScheme};
use crate::config::Config;
use crate::ping::{PingResult, PingStats};
use std::collections::VecDeque;

/// Maximum number of recent pings to track for footer sparkline (enough for wide terminals)
const MAX_RECENT_RTT_COUNT: usize = 500;

/// Popup info for clicked ping
#[derive(Clone)]
pub struct PingPopup {
    /// Stable sequence number of the ping result (not VecDeque index)
    pub result_seq: usize,
    pub screen_x: u16,
    pub screen_y: u16,
}

/// Settings menu field being edited
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Target,
    Interval,
    AutoScale,
    Scale,
    ScaleMin,
    ScaleMax,
    ColorScheme,
    ShowCursor,
    Sparkline,
    DeferRow,
    BufferSize,
    Confirm,
    Cancel,
}

/// Header field that can be edited inline
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeaderEditField {
    Target,
    Interval,
    Scale,
    Colors,
}

/// Focus state within an inline edit popup
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InlineEditFocus {
    /// The text/value input field
    Input,
    /// The secondary text/value input field
    SecondaryInput,
    /// The auto-scale checkbox (only for Scale popup)
    AutoToggle,
    /// The confirm button
    Confirm,
}

impl SettingsField {
    /// Returns true if this field supports text input
    pub fn is_text_input(self) -> bool {
        matches!(
            self,
            SettingsField::Target
                | SettingsField::Interval
                | SettingsField::Scale
                | SettingsField::ScaleMin
                | SettingsField::ScaleMax
                | SettingsField::BufferSize
        )
    }

    /// Returns true if this is a button field
    pub fn is_button(self) -> bool {
        matches!(self, SettingsField::Confirm | SettingsField::Cancel)
    }
}

const MANUAL_SETTINGS_FIELDS: [SettingsField; 11] = [
    SettingsField::Target,
    SettingsField::Interval,
    SettingsField::AutoScale,
    SettingsField::Scale,
    SettingsField::ColorScheme,
    SettingsField::ShowCursor,
    SettingsField::Sparkline,
    SettingsField::DeferRow,
    SettingsField::BufferSize,
    SettingsField::Confirm,
    SettingsField::Cancel,
];

const MANUAL_SETTINGS_CONTENT_FIELDS: [SettingsField; 9] = [
    SettingsField::Target,
    SettingsField::Interval,
    SettingsField::AutoScale,
    SettingsField::Scale,
    SettingsField::ColorScheme,
    SettingsField::ShowCursor,
    SettingsField::Sparkline,
    SettingsField::DeferRow,
    SettingsField::BufferSize,
];

const AUTO_SETTINGS_FIELDS: [SettingsField; 12] = [
    SettingsField::Target,
    SettingsField::Interval,
    SettingsField::AutoScale,
    SettingsField::ScaleMin,
    SettingsField::ScaleMax,
    SettingsField::ColorScheme,
    SettingsField::ShowCursor,
    SettingsField::Sparkline,
    SettingsField::DeferRow,
    SettingsField::BufferSize,
    SettingsField::Confirm,
    SettingsField::Cancel,
];

const AUTO_SETTINGS_CONTENT_FIELDS: [SettingsField; 10] = [
    SettingsField::Target,
    SettingsField::Interval,
    SettingsField::AutoScale,
    SettingsField::ScaleMin,
    SettingsField::ScaleMax,
    SettingsField::ColorScheme,
    SettingsField::ShowCursor,
    SettingsField::Sparkline,
    SettingsField::DeferRow,
    SettingsField::BufferSize,
];

/// Application state
pub struct App {
    pub config: Config,
    pub color_scale: ColorScale,
    pub stats: PingStats,
    pub results: VecDeque<PingResult>,
    /// Maximum history size (calculated from buffer_mb)
    pub max_history: usize,
    /// Base sequence number - total results ever recorded minus current buffer size
    /// Used for stable row calculations when ring buffer wraps
    pub result_base_seq: usize,
    pub should_quit: bool,
    /// Recent RTT values for footer sparkline (ms as f64, None = timeout)
    pub recent_rtts: VecDeque<Option<f64>>,
    /// Whether auto-scale mode is enabled (scale adjusts to visible data)
    pub auto_scale: bool,
    /// Whether to defer rendering of incomplete rows
    pub defer_row: bool,
    /// Whether the display is paused (not recording new pings)
    pub paused: bool,
    /// The row index we're viewing at the BOTTOM of the screen (None = live/follow mode)
    /// When Some(row), we're showing rows ending at `row`, and new data won't shift the view
    pub view_end_row: Option<usize>,
    /// Currently displayed popup (if any)
    pub popup: Option<PingPopup>,
    /// Graph area dimensions for mouse calculations
    pub graph_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
    /// Header area dimensions for mouse calculations
    pub header_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
    /// Footer area dimensions for mouse calculations
    pub footer_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
    /// Whether settings menu is open
    pub settings_open: bool,
    /// Currently selected settings field
    pub settings_field: SettingsField,
    /// Temporary target host being edited
    pub settings_target: String,
    /// Temporary interval value being edited
    pub settings_interval: u64,
    /// Temporary scale value being edited
    pub settings_scale: u64,
    /// Temporary minimum auto-scale value being edited
    pub settings_scale_min: u64,
    /// Temporary maximum auto-scale value being edited
    pub settings_scale_max: u64,
    /// Temporary color scheme being edited
    pub settings_colors: ColorScheme,
    /// Text input buffer for typing values
    pub settings_input_buffer: String,
    /// Cursor position within input buffer
    pub settings_input_cursor: usize,
    /// Whether we're in text input mode
    pub settings_input_active: bool,
    /// Whether the entire input is selected (for select-all behavior)
    pub settings_input_selected: bool,
    /// Original values when settings was opened (for cancel)
    pub settings_original_scale: u64,
    /// Original minimum auto-scale value when settings was opened (for cancel)
    pub settings_original_scale_min: u64,
    /// Original maximum auto-scale value when settings was opened (for cancel)
    pub settings_original_scale_max: u64,
    /// Original color scheme when settings was opened (for cancel)
    pub settings_original_colors: ColorScheme,
    /// Original show cursor when settings was opened (for cancel)
    pub settings_original_show_cursor: bool,
    /// Temporary show cursor value being edited
    pub settings_show_cursor: bool,
    /// Temporary auto_scale value being edited
    pub settings_auto_scale: bool,
    /// Original auto_scale when settings was opened (for cancel)
    pub settings_original_auto_scale: bool,
    /// Temporary sparkline value being edited
    pub settings_sparkline: bool,
    /// Original sparkline when settings was opened (for cancel)
    pub settings_original_sparkline: bool,
    /// Temporary defer_row value being edited
    pub settings_defer_row: bool,
    /// Original defer_row when settings was opened (for cancel)
    pub settings_original_defer_row: bool,
    /// Temporary buffer size being edited (in MB)
    pub settings_buffer_mb: u64,
    /// Whether pinger needs to be restarted (target or interval changed)
    pub needs_pinger_restart: bool,
    /// New target for pinger restart (if changed)
    pub new_target: Option<String>,
    /// New interval for pinger restart (if changed)
    pub new_interval: Option<u64>,
    /// Inline edit popup for header fields
    pub inline_edit: Option<HeaderEditField>,
    /// Inline edit popup position (x, y)
    pub inline_edit_pos: (u16, u16),
    /// Inline edit input buffer
    pub inline_edit_buffer: String,
    /// Inline edit secondary input buffer
    pub inline_edit_secondary_buffer: String,
    /// Inline edit cursor position
    pub inline_edit_cursor: usize,
    /// Inline edit secondary cursor position
    pub inline_edit_secondary_cursor: usize,
    /// Inline edit text selected
    pub inline_edit_selected: bool,
    /// Inline edit secondary text selected
    pub inline_edit_secondary_selected: bool,
    /// Original value before inline edit (for cancel)
    pub inline_edit_original: String,
    /// Original secondary value before inline edit (for cancel)
    pub inline_edit_original_secondary: String,
    /// Whether inline edit is in text input mode (vs navigation mode)
    pub inline_edit_input_active: bool,
    /// Inline edit confirm button area (x, y, width)
    pub inline_edit_confirm_area: Option<(u16, u16, u16)>,
    /// Inline edit auto-toggle area (x, y, width) - only for Scale popup
    pub inline_edit_auto_area: Option<(u16, u16, u16)>,
    /// Which element is focused in inline edit popup
    pub inline_edit_focus: InlineEditFocus,
    /// Current auto_scale value within the inline scale edit
    pub inline_edit_auto_scale: bool,
    /// Original auto_scale value when inline edit opened (for cancel)
    pub inline_edit_original_auto_scale: bool,
    /// Original minimum auto-scale value when inline edit opened (for cancel)
    pub inline_edit_original_scale_min: u64,
    /// Original maximum auto-scale value when inline edit opened (for cancel)
    pub inline_edit_original_scale_max: u64,
    /// Currently selected header field for tab navigation (None = no selection)
    pub header_selected: Option<HeaderEditField>,
    /// Whether quit confirmation dialog is shown
    pub quit_confirm: bool,
    /// Which button is focused in quit dialog (false = Yes, true = No)
    pub quit_confirm_no_focused: bool,
    /// Quit dialog Yes button area (x, y, width)
    pub quit_confirm_yes_area: Option<(u16, u16, u16)>,
    /// Quit dialog No button area (x, y, width)
    pub quit_confirm_no_area: Option<(u16, u16, u16)>,
    /// Legend area dimensions for mouse calculations
    pub legend_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
    /// Currently highlighted RTT range from legend hover (min_rtt, max_rtt, is_timeout)
    /// When Some, graph samples within this range will be highlighted
    pub highlight_rtt_range: Option<(f64, f64, bool)>,
    /// Whether we were in live mode before the popup was shown (to restore when popup closes)
    pub popup_was_live: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let auto_scale = config.scale.is_none();
        let scale_val = config.scale.unwrap_or(200);
        let color_scale = ColorScale::new(scale_val, config.colors);
        let settings_interval = config.interval;
        let settings_scale = scale_val;
        let settings_scale_min = config.scale_min;
        let settings_scale_max = config.scale_max;
        let inline_edit_original_scale_min = config.scale_min;
        let inline_edit_original_scale_max = config.scale_max;
        let settings_colors = config.colors;
        let settings_target = config.host.clone().unwrap_or_default();
        let settings_show_cursor = config.show_cursor;
        let settings_buffer_mb = config.buffer_mb;
        let defer_row = config.defer_row;
        let max_history = config.max_history();
        Self {
            max_history,
            result_base_seq: 0,
            auto_scale,
            config,
            color_scale,
            stats: PingStats::new(),
            results: VecDeque::with_capacity(max_history.min(100000)),
            should_quit: false,
            recent_rtts: VecDeque::with_capacity(MAX_RECENT_RTT_COUNT),
            defer_row,
            paused: false,
            view_end_row: None, // None = live mode (follow newest)
            popup: None,
            graph_area: None,
            header_area: None,
            footer_area: None,
            settings_open: false,
            settings_field: SettingsField::Target,
            settings_target,
            settings_interval,
            settings_scale,
            settings_scale_min,
            settings_scale_max,
            settings_colors,
            settings_input_buffer: String::new(),
            settings_input_cursor: 0,
            settings_input_active: false,
            settings_input_selected: false,
            settings_original_scale: settings_scale,
            settings_original_scale_min: settings_scale_min,
            settings_original_scale_max: settings_scale_max,
            settings_original_colors: settings_colors,
            settings_original_show_cursor: settings_show_cursor,
            settings_show_cursor,
            settings_auto_scale: auto_scale,
            settings_original_auto_scale: auto_scale,
            settings_sparkline: false,
            settings_original_sparkline: false,
            settings_defer_row: defer_row,
            settings_original_defer_row: defer_row,
            settings_buffer_mb,
            needs_pinger_restart: false,
            new_target: None,
            new_interval: None,
            inline_edit: None,
            inline_edit_pos: (0, 0),
            inline_edit_buffer: String::new(),
            inline_edit_secondary_buffer: String::new(),
            inline_edit_cursor: 0,
            inline_edit_secondary_cursor: 0,
            inline_edit_selected: false,
            inline_edit_secondary_selected: false,
            inline_edit_original: String::new(),
            inline_edit_original_secondary: String::new(),
            inline_edit_input_active: false,
            inline_edit_confirm_area: None,
            inline_edit_auto_area: None,
            inline_edit_focus: InlineEditFocus::Input,
            inline_edit_auto_scale: auto_scale,
            inline_edit_original_auto_scale: auto_scale,
            inline_edit_original_scale_min,
            inline_edit_original_scale_max,
            header_selected: None,
            quit_confirm: false,
            quit_confirm_no_focused: false,
            quit_confirm_yes_area: None,
            quit_confirm_no_area: None,
            legend_area: None,
            highlight_rtt_range: None,
            popup_was_live: false,
        }
    }

    pub fn record_result(&mut self, result: PingResult) {
        self.stats.record(&result);

        // Track recent RTT for sparkline
        let rtt_ms = result.rtt_ms_f64();
        self.recent_rtts.push_back(rtt_ms);
        while self.recent_rtts.len() > MAX_RECENT_RTT_COUNT {
            self.recent_rtts.pop_front();
        }

        self.results.push_back(result);

        // Keep history bounded to max_history
        while self.results.len() > self.max_history {
            self.results.pop_front();
            self.result_base_seq += 1;
        }
    }

    /// Get recent RTTs as a slice for the footer
    pub fn recent_rtts_slice(&self) -> Vec<Option<f64>> {
        self.recent_rtts.iter().cloned().collect()
    }

    /// Clear all stats, results, and history (used when target changes)
    pub fn clear_all_data(&mut self) {
        self.stats = PingStats::new();
        self.results.clear();
        self.recent_rtts.clear();
        self.result_base_seq = 0;
        self.view_end_row = None;
        self.popup = None;
    }

    /// Compute the maximum RTT (ms) among visible data points on the graph.
    /// Returns a suitable scale value with headroom, or a default if no data.
    pub fn compute_visible_max_rtt(&self, width: usize, height: usize) -> u64 {
        if self.results.is_empty() || width == 0 || height == 0 {
            return self.bounded_auto_scale_value(self.color_scale.max_rtt);
        }

        let total_results = self.result_base_seq + self.results.len();
        let total_rows = total_results.div_ceil(width);
        let view_end = self.view_end_row.unwrap_or(total_rows).min(total_rows);
        let visible_rows = view_end.min(height);
        let view_start_row = view_end.saturating_sub(visible_rows);

        let start_seq = view_start_row * width;
        let end_seq = (view_end * width).min(total_results);

        let mut max_rtt_ms = 0.0f64;
        let mut found_any = false;
        for seq_idx in start_seq..end_seq {
            if seq_idx < self.result_base_seq {
                continue;
            }
            let vec_idx = seq_idx - self.result_base_seq;
            if vec_idx < self.results.len()
                && let Some(rtt) = self.results[vec_idx].rtt_ms_f64()
            {
                max_rtt_ms = max_rtt_ms.max(rtt);
                found_any = true;
            }
        }

        if !found_any {
            return self.color_scale.max_rtt;
        }

        // Add 20% headroom and round up
        let with_headroom = (max_rtt_ms * 1.2).ceil() as u64;
        self.bounded_auto_scale_value(with_headroom.max(1))
    }

    fn bounded_auto_scale_value(&self, value: u64) -> u64 {
        value.clamp(self.config.scale_min, self.config.scale_max)
    }

    fn settings_visible_fields(&self) -> &'static [SettingsField] {
        if self.settings_auto_scale {
            &AUTO_SETTINGS_FIELDS
        } else {
            &MANUAL_SETTINGS_FIELDS
        }
    }

    fn settings_content_fields(&self) -> &'static [SettingsField] {
        if self.settings_auto_scale {
            &AUTO_SETTINGS_CONTENT_FIELDS
        } else {
            &MANUAL_SETTINGS_CONTENT_FIELDS
        }
    }

    fn settings_menu_height(&self) -> u16 {
        let field_count = self.settings_content_fields().len() as u16;
        field_count * 2 + 7
    }

    fn normalize_settings_scale_bounds(&mut self, changed_field: SettingsField) {
        match changed_field {
            SettingsField::ScaleMin if self.settings_scale_min > self.settings_scale_max => {
                self.settings_scale_max = self.settings_scale_min;
            }
            SettingsField::ScaleMax if self.settings_scale_max < self.settings_scale_min => {
                self.settings_scale_min = self.settings_scale_max;
            }
            _ => {}
        }

        self.config.scale_min = self.settings_scale_min;
        self.config.scale_max = self.settings_scale_max;

        if self.settings_auto_scale {
            let bounded = self.bounded_auto_scale_value(self.color_scale.max_rtt);
            self.color_scale = ColorScale::new(bounded, self.settings_colors);
        }
    }

    fn sync_settings_field_visibility(&mut self) {
        if self
            .settings_visible_fields()
            .contains(&self.settings_field)
        {
            return;
        }

        self.settings_input_active = false;
        self.settings_input_selected = false;
        self.settings_field = if self.settings_auto_scale {
            SettingsField::ScaleMin
        } else {
            SettingsField::Scale
        };
    }

    /// Calculate current total rows of data (using stable sequence numbers)
    pub fn total_rows(&self, width: usize) -> usize {
        if width == 0 || self.results.is_empty() {
            return 0;
        }
        // Use base_seq + results.len() for stable row calculation
        let total_results = self.result_base_seq + self.results.len();
        total_results.div_ceil(width)
    }

    /// Compute min/avg/max RTT stats for the data points currently visible on the graph.
    /// Returns (min, avg, max) as formatted strings in ms, or "-" if no data.
    pub fn compute_visible_stats(&self, width: usize, height: usize) -> (String, String, String) {
        let no_data = ("-".to_string(), "-".to_string(), "-".to_string());
        if self.results.is_empty() || width == 0 || height == 0 {
            return no_data;
        }

        let total_results = self.result_base_seq + self.results.len();
        let total_rows = total_results.div_ceil(width);
        let view_end = self.view_end_row.unwrap_or(total_rows).min(total_rows);
        let visible_rows = view_end.min(height);
        let view_start_row = view_end.saturating_sub(visible_rows);

        let start_seq = view_start_row * width;
        let end_seq = (view_end * width).min(total_results);

        let mut min_rtt = f64::INFINITY;
        let mut max_rtt = 0.0f64;
        let mut sum = 0.0f64;
        let mut count = 0usize;

        for seq_idx in start_seq..end_seq {
            if seq_idx < self.result_base_seq {
                continue;
            }
            let vec_idx = seq_idx - self.result_base_seq;
            if vec_idx < self.results.len()
                && let Some(rtt) = self.results[vec_idx].rtt_ms_f64()
            {
                min_rtt = min_rtt.min(rtt);
                max_rtt = max_rtt.max(rtt);
                sum += rtt;
                count += 1;
            }
        }

        if count == 0 {
            return no_data;
        }

        (
            format!("{:.2}", min_rtt),
            format!("{:.2}", sum / count as f64),
            format!("{:.2}", max_rtt),
        )
    }

    /// Get the current view end row (for display purposes)
    #[allow(dead_code)]
    pub fn current_view_end_row(&self, width: usize) -> usize {
        match self.view_end_row {
            Some(row) => row,
            None => self.total_rows(width), // Live mode: show latest
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if !self.paused {
            // Resume: jump to live view
            self.view_end_row = None;
        }
    }

    pub fn scroll_up(&mut self, rows: usize) {
        if let Some((_, _, width, _)) = self.graph_area {
            let width = width as usize;
            let total_rows = self.total_rows(width);

            if total_rows == 0 {
                return;
            }

            // Get current view end
            let current_end = self.view_end_row.unwrap_or(total_rows);

            // Scroll up means showing older data (lower row numbers)
            // Allow scrolling all the way to row 1 (view_end_row = 1) so only first row is visible
            let new_end = current_end.saturating_sub(rows).max(1);

            self.view_end_row = Some(new_end);
            // Note: scrolling doesn't pause - pings keep collecting, just view is locked
        }
    }

    pub fn scroll_down(&mut self, rows: usize) {
        if let Some((_, _, width, _)) = self.graph_area {
            let width = width as usize;
            let total_rows = self.total_rows(width);

            if let Some(current_end) = self.view_end_row {
                let new_end = current_end + rows;

                if new_end >= total_rows {
                    // Reached the end, switch to live mode
                    self.view_end_row = None;
                } else {
                    self.view_end_row = Some(new_end);
                }
            }
            // If already in live mode, do nothing
        }
    }

    pub fn jump_to_live(&mut self) {
        self.view_end_row = None;
        self.paused = false;
    }

    /// Check if we're in live mode (following newest data)
    #[allow(dead_code)]
    pub fn is_live(&self) -> bool {
        self.view_end_row.is_none()
    }

    /// Get the PingResult at a given index if it exists
    #[allow(dead_code)]
    pub fn get_result(&self, idx: usize) -> Option<&PingResult> {
        self.results.get(idx)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Open settings menu (unconditional)
    pub fn open_settings(&mut self) {
        self.settings_open = true;
        // Always start with the first field selected
        self.settings_field = SettingsField::Target;
        // Initialize with current active values (not startup values)
        self.settings_target = self.config.host.clone().unwrap_or_default();
        self.settings_interval = self.config.interval;
        self.settings_scale = self.color_scale.max_rtt;
        self.settings_scale_min = self.config.scale_min;
        self.settings_scale_max = self.config.scale_max;
        self.settings_colors = self.color_scale.scheme;
        self.settings_show_cursor = self.config.show_cursor;
        self.settings_input_buffer.clear();
        self.settings_input_cursor = 0;
        self.settings_input_active = false;
        self.settings_input_selected = false;
        // Store originals for cancel
        self.settings_original_scale = self.color_scale.max_rtt;
        self.settings_original_scale_min = self.config.scale_min;
        self.settings_original_scale_max = self.config.scale_max;
        self.settings_original_colors = self.color_scale.scheme;
        self.settings_original_show_cursor = self.config.show_cursor;
        self.settings_original_auto_scale = self.auto_scale;
        self.settings_original_sparkline = self.config.sparkline;
        self.settings_original_defer_row = self.defer_row;
        self.settings_auto_scale = self.auto_scale;
        self.settings_sparkline = self.config.sparkline;
        self.settings_defer_row = self.defer_row;
        self.config.scale_min = self.settings_scale_min;
        self.config.scale_max = self.settings_scale_max;
    }

    /// Toggle settings menu
    pub fn toggle_settings(&mut self) {
        if self.settings_open {
            self.settings_open = false;
        } else {
            self.open_settings();
        }
    }

    /// Cancel settings and restore original values
    pub fn cancel_settings(&mut self) {
        // Restore original values
        self.color_scale =
            ColorScale::new(self.settings_original_scale, self.settings_original_colors);
        self.config.show_cursor = self.settings_original_show_cursor;
        self.auto_scale = self.settings_original_auto_scale;
        self.config.scale_min = self.settings_original_scale_min;
        self.config.scale_max = self.settings_original_scale_max;
        self.config.sparkline = self.settings_original_sparkline;
        self.defer_row = self.settings_original_defer_row;
        self.config.defer_row = self.settings_original_defer_row;
        self.settings_open = false;
        self.settings_input_active = false;
    }

    /// Apply settings changes
    pub fn apply_settings(&mut self) {
        // Check if target changed
        let current_target = self.config.host.clone().unwrap_or_default();
        let target_changed =
            !self.settings_target.is_empty() && self.settings_target != current_target;

        // Check if interval changed
        let interval_changed = self.settings_interval != self.config.interval;

        // Apply target
        if !self.settings_target.is_empty() {
            self.config.host = Some(self.settings_target.clone());
        }
        // Apply interval
        self.config.interval = self.settings_interval;
        // Apply scale and colors
        self.config.scale = if self.settings_auto_scale {
            None
        } else {
            Some(self.settings_scale)
        };
        self.config.scale_min = self.settings_scale_min;
        self.config.scale_max = self.settings_scale_max;
        self.config.colors = self.settings_colors;
        let effective_scale = if self.settings_auto_scale {
            self.bounded_auto_scale_value(self.color_scale.max_rtt)
        } else {
            self.settings_scale
        };
        self.color_scale = ColorScale::new(effective_scale, self.settings_colors);
        // Apply show cursor
        self.config.show_cursor = self.settings_show_cursor;
        // Apply auto scale
        self.auto_scale = self.settings_auto_scale;
        // Apply sparkline
        self.config.sparkline = self.settings_sparkline;
        // Apply defer row
        self.defer_row = self.settings_defer_row;
        self.config.defer_row = self.settings_defer_row;
        // Apply buffer size
        self.config.buffer_mb = self.settings_buffer_mb;
        self.max_history = self.config.max_history();

        // Signal pinger restart if target or interval changed
        if target_changed || interval_changed {
            self.needs_pinger_restart = true;
            if target_changed {
                self.new_target = Some(self.settings_target.clone());
            }
            if interval_changed {
                self.new_interval = Some(self.settings_interval);
            }
        }
    }

    /// Navigate to next settings field
    pub fn settings_next_field(&mut self) {
        let fields = self.settings_visible_fields();
        let next_index = fields
            .iter()
            .position(|field| *field == self.settings_field)
            .map(|index| (index + 1) % fields.len())
            .unwrap_or(0);
        self.settings_field = fields[next_index];
    }

    /// Navigate to previous settings field
    pub fn settings_prev_field(&mut self) {
        let fields = self.settings_visible_fields();
        let prev_index = fields
            .iter()
            .position(|field| *field == self.settings_field)
            .map(|index| (index + fields.len() - 1) % fields.len())
            .unwrap_or(fields.len().saturating_sub(1));
        self.settings_field = fields[prev_index];
    }

    /// Increase current settings value
    pub fn settings_increase(&mut self) {
        match self.settings_field {
            SettingsField::Target | SettingsField::Confirm | SettingsField::Cancel => {}
            SettingsField::Interval => {
                self.settings_interval = self.settings_interval.saturating_add(1).min(100000);
            }
            SettingsField::Scale => {
                self.settings_scale = self.settings_scale.saturating_add(1).min(100000);
                // Apply immediately
                self.color_scale = ColorScale::new(self.settings_scale, self.settings_colors);
            }
            SettingsField::ScaleMin => {
                self.settings_scale_min = self.settings_scale_min.saturating_add(1).min(100000);
                self.normalize_settings_scale_bounds(SettingsField::ScaleMin);
            }
            SettingsField::ScaleMax => {
                self.settings_scale_max = self.settings_scale_max.saturating_add(1).min(100000);
                self.normalize_settings_scale_bounds(SettingsField::ScaleMax);
            }
            SettingsField::AutoScale => {
                self.settings_auto_scale = !self.settings_auto_scale;
                self.auto_scale = self.settings_auto_scale;
                self.sync_settings_field_visibility();
            }
            SettingsField::ColorScheme => {
                self.settings_colors = self.settings_colors.next();
                // Apply immediately
                self.color_scale = ColorScale::new(self.settings_scale, self.settings_colors);
            }
            SettingsField::ShowCursor => {
                self.settings_show_cursor = !self.settings_show_cursor;
                // Apply immediately for live preview
                self.config.show_cursor = self.settings_show_cursor;
            }
            SettingsField::Sparkline => {
                self.settings_sparkline = !self.settings_sparkline;
                self.config.sparkline = self.settings_sparkline;
            }
            SettingsField::DeferRow => {
                self.settings_defer_row = !self.settings_defer_row;
                self.defer_row = self.settings_defer_row;
                self.config.defer_row = self.settings_defer_row;
            }
            SettingsField::BufferSize => {
                self.settings_buffer_mb = self.settings_buffer_mb.saturating_add(1).min(1000);
            }
        }
    }

    /// Decrease current settings value
    pub fn settings_decrease(&mut self) {
        match self.settings_field {
            SettingsField::Target | SettingsField::Confirm | SettingsField::Cancel => {}
            SettingsField::Interval => {
                self.settings_interval = self.settings_interval.saturating_sub(1).max(1);
            }
            SettingsField::Scale => {
                self.settings_scale = self.settings_scale.saturating_sub(1).max(1);
                // Apply immediately
                self.color_scale = ColorScale::new(self.settings_scale, self.settings_colors);
            }
            SettingsField::ScaleMin => {
                self.settings_scale_min = self.settings_scale_min.saturating_sub(1).max(1);
                self.normalize_settings_scale_bounds(SettingsField::ScaleMin);
            }
            SettingsField::ScaleMax => {
                self.settings_scale_max = self.settings_scale_max.saturating_sub(1).max(1);
                self.normalize_settings_scale_bounds(SettingsField::ScaleMax);
            }
            SettingsField::AutoScale => {
                self.settings_auto_scale = !self.settings_auto_scale;
                self.auto_scale = self.settings_auto_scale;
                self.sync_settings_field_visibility();
            }
            SettingsField::ColorScheme => {
                self.settings_colors = self.settings_colors.prev();
                // Apply immediately
                self.color_scale = ColorScale::new(self.settings_scale, self.settings_colors);
            }
            SettingsField::ShowCursor => {
                self.settings_show_cursor = !self.settings_show_cursor;
                // Apply immediately for live preview
                self.config.show_cursor = self.settings_show_cursor;
            }
            SettingsField::Sparkline => {
                self.settings_sparkline = !self.settings_sparkline;
                self.config.sparkline = self.settings_sparkline;
            }
            SettingsField::DeferRow => {
                self.settings_defer_row = !self.settings_defer_row;
                self.defer_row = self.settings_defer_row;
                self.config.defer_row = self.settings_defer_row;
            }
            SettingsField::BufferSize => {
                self.settings_buffer_mb = self.settings_buffer_mb.saturating_sub(1).max(1);
            }
        }
    }

    /// Start text input mode for current field
    pub fn settings_start_input(&mut self) {
        if self.settings_field.is_text_input() {
            self.settings_input_active = true;
            self.settings_input_selected = true; // Select all on entry
            self.settings_input_buffer = match self.settings_field {
                SettingsField::Target => self.settings_target.clone(),
                SettingsField::Interval => self.settings_interval.to_string(),
                SettingsField::Scale => self.settings_scale.to_string(),
                SettingsField::ScaleMin => self.settings_scale_min.to_string(),
                SettingsField::ScaleMax => self.settings_scale_max.to_string(),
                SettingsField::BufferSize => self.settings_buffer_mb.to_string(),
                SettingsField::ColorScheme
                | SettingsField::ShowCursor
                | SettingsField::AutoScale
                | SettingsField::Sparkline
                | SettingsField::DeferRow
                | SettingsField::Confirm
                | SettingsField::Cancel => String::new(),
            };
            self.settings_input_cursor = self.settings_input_buffer.len();
        }
    }

    /// Handle character input in text mode
    pub fn settings_input_char(&mut self, c: char) {
        if self.settings_input_active {
            // If text is selected, clear it and start fresh
            if self.settings_input_selected {
                self.settings_input_buffer.clear();
                self.settings_input_cursor = 0;
                self.settings_input_selected = false;
            }

            match self.settings_field {
                SettingsField::Target => {
                    self.settings_input_buffer
                        .insert(self.settings_input_cursor, c);
                    self.settings_input_cursor += 1;
                    self.settings_target = self.settings_input_buffer.clone();
                }
                SettingsField::Interval
                | SettingsField::Scale
                | SettingsField::ScaleMin
                | SettingsField::ScaleMax
                | SettingsField::BufferSize => {
                    if c.is_ascii_digit() {
                        self.settings_input_buffer
                            .insert(self.settings_input_cursor, c);
                        self.settings_input_cursor += 1;
                        if let Ok(val) = self.settings_input_buffer.parse::<u64>() {
                            let clamped = val.clamp(1, 100000);
                            match self.settings_field {
                                SettingsField::Interval => self.settings_interval = clamped,
                                SettingsField::Scale => {
                                    self.settings_scale = clamped;
                                    self.color_scale =
                                        ColorScale::new(self.settings_scale, self.settings_colors);
                                }
                                SettingsField::ScaleMin => {
                                    self.settings_scale_min = clamped;
                                    self.normalize_settings_scale_bounds(SettingsField::ScaleMin);
                                }
                                SettingsField::ScaleMax => {
                                    self.settings_scale_max = clamped;
                                    self.normalize_settings_scale_bounds(SettingsField::ScaleMax);
                                }
                                SettingsField::BufferSize => {
                                    self.settings_buffer_mb = clamped;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                SettingsField::ColorScheme
                | SettingsField::ShowCursor
                | SettingsField::AutoScale
                | SettingsField::Sparkline
                | SettingsField::DeferRow
                | SettingsField::Confirm
                | SettingsField::Cancel => {}
            }
        }
    }

    /// Handle backspace in text mode
    pub fn settings_input_backspace(&mut self) {
        if self.settings_input_active {
            // If text is selected, clear all
            if self.settings_input_selected {
                self.settings_input_buffer.clear();
                self.settings_input_cursor = 0;
                self.settings_input_selected = false;
            } else if self.settings_input_cursor > 0 {
                self.settings_input_cursor -= 1;
                self.settings_input_buffer
                    .remove(self.settings_input_cursor);
            }

            match self.settings_field {
                SettingsField::Target => {
                    self.settings_target = self.settings_input_buffer.clone();
                }
                SettingsField::Interval => {
                    self.settings_interval = self.settings_input_buffer.parse().unwrap_or(1).max(1);
                }
                SettingsField::Scale => {
                    self.settings_scale = self.settings_input_buffer.parse().unwrap_or(1).max(1);
                    self.color_scale = ColorScale::new(self.settings_scale, self.settings_colors);
                }
                SettingsField::ScaleMin => {
                    self.settings_scale_min =
                        self.settings_input_buffer.parse().unwrap_or(1).max(1);
                    self.normalize_settings_scale_bounds(SettingsField::ScaleMin);
                }
                SettingsField::ScaleMax => {
                    self.settings_scale_max =
                        self.settings_input_buffer.parse().unwrap_or(1).max(1);
                    self.normalize_settings_scale_bounds(SettingsField::ScaleMax);
                }
                SettingsField::BufferSize => {
                    self.settings_buffer_mb =
                        self.settings_input_buffer.parse().unwrap_or(1).max(1);
                }
                SettingsField::ColorScheme
                | SettingsField::ShowCursor
                | SettingsField::AutoScale
                | SettingsField::Sparkline
                | SettingsField::DeferRow
                | SettingsField::Confirm
                | SettingsField::Cancel => {}
            }
        }
    }

    /// Move cursor left in text input mode
    pub fn settings_input_left(&mut self) {
        if self.settings_input_active {
            self.settings_input_selected = false;
            if self.settings_input_cursor > 0 {
                self.settings_input_cursor -= 1;
            }
        }
    }

    /// Move cursor right in text input mode
    pub fn settings_input_right(&mut self) {
        if self.settings_input_active {
            self.settings_input_selected = false;
            if self.settings_input_cursor < self.settings_input_buffer.len() {
                self.settings_input_cursor += 1;
            }
        }
    }

    /// Confirm text input
    pub fn settings_confirm_input(&mut self) {
        self.settings_input_active = false;
        self.settings_input_selected = false;
    }

    /// Handle mouse click in settings menu
    /// Returns true if click was handled (inside settings area)
    pub fn settings_handle_click(
        &mut self,
        screen_x: u16,
        screen_y: u16,
        area_width: u16,
        area_height: u16,
    ) -> bool {
        // Calculate settings menu position (same as in SettingsMenu render)
        let menu_width = 65u16.min(area_width.saturating_sub(4));
        let menu_height = self
            .settings_menu_height()
            .min(area_height.saturating_sub(4));
        let menu_x = (area_width.saturating_sub(menu_width)) / 2;
        let menu_y = (area_height.saturating_sub(menu_height)) / 2;

        // Check if click is inside menu
        if screen_x < menu_x
            || screen_x >= menu_x + menu_width
            || screen_y < menu_y
            || screen_y >= menu_y + menu_height
        {
            return false;
        }

        // Convert to relative coordinates within menu (accounting for border)
        let rel_x = screen_x.saturating_sub(menu_x + 1); // +1 for border
        let rel_y = screen_y.saturating_sub(menu_y + 1); // +1 for border

        let content_fields = self.settings_content_fields();
        let buttons_row = 1 + (content_fields.len() as u16 * 2);

        let clicked_field = if rel_y == buttons_row {
            // Buttons row - check x position
            // "                    " (20 spaces) + " Confirm " (9) + "    " (4) + " Cancel " (8)
            // Confirm: x 20-28, Cancel: x 33-40
            if (20..29).contains(&rel_x) {
                Some(SettingsField::Confirm)
            } else if (33..41).contains(&rel_x) {
                Some(SettingsField::Cancel)
            } else {
                None
            }
        } else if rel_y % 2 == 1 {
            content_fields.get(((rel_y - 1) / 2) as usize).copied()
        } else {
            None
        };

        if let Some(field) = clicked_field {
            // If clicking on currently selected field
            if field == self.settings_field {
                // Activate based on field type
                match field {
                    SettingsField::Target
                    | SettingsField::Interval
                    | SettingsField::Scale
                    | SettingsField::ScaleMin
                    | SettingsField::ScaleMax
                    | SettingsField::BufferSize => {
                        if !self.settings_input_active {
                            self.settings_start_input();
                        }
                    }
                    SettingsField::ColorScheme => {
                        self.settings_increase();
                    }
                    SettingsField::ShowCursor
                    | SettingsField::AutoScale
                    | SettingsField::Sparkline
                    | SettingsField::DeferRow => {
                        self.settings_increase();
                    }
                    SettingsField::Confirm => {
                        self.apply_settings();
                        self.settings_open = false;
                    }
                    SettingsField::Cancel => {
                        self.cancel_settings();
                    }
                }
            } else {
                // Select the field
                self.settings_input_active = false;
                self.settings_field = field;
                // Auto-start input for text fields on first click
                if field.is_text_input() {
                    self.settings_start_input();
                }
            }
        }

        true
    }

    fn parse_scale_input(buffer: &str) -> u64 {
        buffer.parse::<u64>().unwrap_or(1).clamp(1, 100000)
    }

    pub fn inline_edit_has_secondary_input(&self) -> bool {
        self.inline_edit == Some(HeaderEditField::Scale) && self.inline_edit_auto_scale
    }

    pub fn inline_edit_toggle_auto_scale(&mut self) {
        if self.inline_edit != Some(HeaderEditField::Scale) {
            return;
        }

        self.inline_edit_auto_scale = !self.inline_edit_auto_scale;
        self.auto_scale = self.inline_edit_auto_scale;

        if self.inline_edit_auto_scale {
            self.inline_edit_buffer = self.config.scale_min.to_string();
            self.inline_edit_secondary_buffer = self.config.scale_max.to_string();
            self.inline_edit_cursor = self.inline_edit_buffer.len();
            self.inline_edit_secondary_cursor = self.inline_edit_secondary_buffer.len();
            self.inline_edit_selected = true;
            self.inline_edit_secondary_selected = true;
            self.inline_edit_focus = InlineEditFocus::Input;
            self.inline_edit_input_active = true;
        } else {
            let manual_scale = self.bounded_auto_scale_value(self.color_scale.max_rtt);
            self.inline_edit_buffer = manual_scale.to_string();
            self.inline_edit_secondary_buffer.clear();
            self.inline_edit_cursor = self.inline_edit_buffer.len();
            self.inline_edit_secondary_cursor = 0;
            self.inline_edit_selected = true;
            self.inline_edit_secondary_selected = false;
            self.inline_edit_focus = InlineEditFocus::Input;
            self.inline_edit_input_active = true;
            self.color_scale = ColorScale::new(manual_scale, self.color_scale.scheme);
        }

        self.sync_inline_scale_preview();
    }

    fn sync_inline_scale_preview(&mut self) {
        if self.inline_edit != Some(HeaderEditField::Scale) {
            return;
        }

        if self.inline_edit_auto_scale {
            let mut min_scale = Self::parse_scale_input(&self.inline_edit_buffer);
            let mut max_scale = Self::parse_scale_input(&self.inline_edit_secondary_buffer);

            match self.inline_edit_focus {
                InlineEditFocus::SecondaryInput if max_scale < min_scale => {
                    min_scale = max_scale;
                    self.inline_edit_buffer = min_scale.to_string();
                    self.inline_edit_cursor =
                        self.inline_edit_cursor.min(self.inline_edit_buffer.len());
                }
                _ if min_scale > max_scale => {
                    max_scale = min_scale;
                    self.inline_edit_secondary_buffer = max_scale.to_string();
                    self.inline_edit_secondary_cursor = self
                        .inline_edit_secondary_cursor
                        .min(self.inline_edit_secondary_buffer.len());
                }
                _ => {}
            }

            self.config.scale_min = min_scale;
            self.config.scale_max = max_scale;

            let bounded = self.bounded_auto_scale_value(self.color_scale.max_rtt);
            self.color_scale = ColorScale::new(bounded, self.color_scale.scheme);
        } else {
            let manual_scale = Self::parse_scale_input(&self.inline_edit_buffer);
            self.color_scale = ColorScale::new(manual_scale, self.color_scale.scheme);
        }
    }

    /// Start inline edit for a header field
    pub fn start_inline_edit(&mut self, field: HeaderEditField, x: u16, y: u16) {
        self.inline_edit = Some(field);
        self.inline_edit_pos = (x, y);
        self.inline_edit_buffer = match field {
            HeaderEditField::Target => self.config.host.clone().unwrap_or_default(),
            HeaderEditField::Interval => self.config.interval.to_string(),
            HeaderEditField::Scale => {
                if self.auto_scale {
                    self.config.scale_min.to_string()
                } else {
                    self.color_scale.max_rtt.to_string()
                }
            }
            HeaderEditField::Colors => format!("{}", self.color_scale.scheme),
        };
        self.inline_edit_secondary_buffer = if field == HeaderEditField::Scale && self.auto_scale {
            self.config.scale_max.to_string()
        } else {
            String::new()
        };
        self.inline_edit_original = self.inline_edit_buffer.clone();
        self.inline_edit_original_secondary = self.inline_edit_secondary_buffer.clone();
        // For text fields, start with text selected and input active
        if field != HeaderEditField::Colors {
            self.inline_edit_input_active = true;
            self.inline_edit_selected = true;
            self.inline_edit_cursor = self.inline_edit_buffer.len();
            self.inline_edit_secondary_cursor = self.inline_edit_secondary_buffer.len();
            self.inline_edit_secondary_selected = true;
        } else {
            self.inline_edit_input_active = false;
            self.inline_edit_selected = false;
            self.inline_edit_cursor = 0;
            self.inline_edit_secondary_cursor = 0;
            self.inline_edit_secondary_selected = false;
        }
        // Clear header selection when opening inline edit
        self.header_selected = None;
        // Start with input focused
        self.inline_edit_focus = InlineEditFocus::Input;
        // Store auto_scale state for Scale popup
        self.inline_edit_auto_scale = self.auto_scale;
        self.inline_edit_original_auto_scale = self.auto_scale;
        self.inline_edit_original_scale_min = self.config.scale_min;
        self.inline_edit_original_scale_max = self.config.scale_max;
    }

    /// Activate text input mode for inline edit
    pub fn inline_edit_activate_input(&mut self) {
        if let Some(field) = self.inline_edit {
            // Don't activate input mode for Colors (it's an enum selector)
            if field != HeaderEditField::Colors {
                self.inline_edit_input_active = true;
                match self.inline_edit_focus {
                    InlineEditFocus::SecondaryInput => {
                        self.inline_edit_secondary_selected = true;
                        self.inline_edit_secondary_cursor = self.inline_edit_secondary_buffer.len();
                    }
                    _ => {
                        self.inline_edit_selected = true;
                        self.inline_edit_cursor = self.inline_edit_buffer.len();
                    }
                }
            }
        }
    }

    /// Cancel inline edit
    pub fn cancel_inline_edit(&mut self) {
        if let Some(field) = self.inline_edit {
            // Restore original value for scale/colors (they have live preview)
            match field {
                HeaderEditField::Scale => {
                    self.config.scale_min = self.inline_edit_original_scale_min;
                    self.config.scale_max = self.inline_edit_original_scale_max;
                    if self.inline_edit_original_auto_scale {
                        let bounded = self.bounded_auto_scale_value(self.color_scale.max_rtt);
                        self.color_scale = ColorScale::new(bounded, self.color_scale.scheme);
                    } else if let Ok(val) = self.inline_edit_original.parse::<u64>() {
                        self.color_scale = ColorScale::new(val.max(1), self.color_scale.scheme);
                    }
                    self.auto_scale = self.inline_edit_original_auto_scale;
                }
                HeaderEditField::Colors => {
                    // Find original color scheme by cycling through
                    let mut scheme = ColorScheme::default();
                    for _ in 0..10 {
                        if format!("{}", scheme) == self.inline_edit_original {
                            self.color_scale = ColorScale::new(self.color_scale.max_rtt, scheme);
                            break;
                        }
                        scheme = scheme.next();
                    }
                }
                _ => {}
            }
        }
        self.inline_edit = None;
        self.inline_edit_input_active = false;
        self.inline_edit_secondary_selected = false;
    }

    /// Apply inline edit
    pub fn apply_inline_edit(&mut self) {
        if let Some(field) = self.inline_edit {
            match field {
                HeaderEditField::Target => {
                    if !self.inline_edit_buffer.is_empty()
                        && self.inline_edit_buffer != self.config.host.clone().unwrap_or_default()
                    {
                        self.config.host = Some(self.inline_edit_buffer.clone());
                        self.new_target = Some(self.inline_edit_buffer.clone());
                        self.needs_pinger_restart = true;
                    }
                }
                HeaderEditField::Interval => {
                    if let Ok(val) = self.inline_edit_buffer.parse::<u64>() {
                        let clamped = val.clamp(1, 100000);
                        if clamped != self.config.interval {
                            self.config.interval = clamped;
                            self.new_interval = Some(clamped);
                            self.needs_pinger_restart = true;
                        }
                    }
                }
                HeaderEditField::Scale => {
                    // Apply the auto_scale setting from the popup
                    self.auto_scale = self.inline_edit_auto_scale;
                    if self.auto_scale {
                        self.config.scale = None;
                        self.config.scale_min = Self::parse_scale_input(&self.inline_edit_buffer);
                        self.config.scale_max =
                            Self::parse_scale_input(&self.inline_edit_secondary_buffer)
                                .max(self.config.scale_min);
                        let bounded = self.bounded_auto_scale_value(self.color_scale.max_rtt);
                        self.color_scale = ColorScale::new(bounded, self.color_scale.scheme);
                    } else {
                        let manual_scale = Self::parse_scale_input(&self.inline_edit_buffer);
                        self.config.scale = Some(manual_scale);
                        self.color_scale = ColorScale::new(manual_scale, self.color_scale.scheme);
                    }
                }
                HeaderEditField::Colors => {
                    // Already applied via live preview
                    self.config.colors = self.color_scale.scheme;
                }
            }
        }
        self.inline_edit = None;
        self.inline_edit_input_active = false;
    }

    /// Handle character input in inline edit
    pub fn inline_edit_char(&mut self, c: char) {
        if self.inline_edit.is_none() {
            return;
        }
        let field = self.inline_edit.unwrap();

        match field {
            HeaderEditField::Target => {
                if self.inline_edit_selected {
                    self.inline_edit_buffer.clear();
                    self.inline_edit_cursor = 0;
                    self.inline_edit_selected = false;
                }
                self.inline_edit_buffer.insert(self.inline_edit_cursor, c);
                self.inline_edit_cursor += 1;
            }
            HeaderEditField::Interval | HeaderEditField::Scale => {
                if c.is_ascii_digit() {
                    let (buffer, cursor, selected) = match self.inline_edit_focus {
                        InlineEditFocus::SecondaryInput => (
                            &mut self.inline_edit_secondary_buffer,
                            &mut self.inline_edit_secondary_cursor,
                            &mut self.inline_edit_secondary_selected,
                        ),
                        _ => (
                            &mut self.inline_edit_buffer,
                            &mut self.inline_edit_cursor,
                            &mut self.inline_edit_selected,
                        ),
                    };

                    if *selected {
                        buffer.clear();
                        *cursor = 0;
                        *selected = false;
                    }

                    buffer.insert(*cursor, c);
                    *cursor += 1;

                    if field == HeaderEditField::Scale {
                        self.sync_inline_scale_preview();
                    }
                }
            }
            HeaderEditField::Colors => {
                // Colors don't accept text input - use scroll wheel
            }
        }
    }

    /// Handle backspace in inline edit
    pub fn inline_edit_backspace(&mut self) {
        if self.inline_edit.is_none() {
            return;
        }
        let field = self.inline_edit.unwrap();

        let (buffer, cursor, selected) = match self.inline_edit_focus {
            InlineEditFocus::SecondaryInput => (
                &mut self.inline_edit_secondary_buffer,
                &mut self.inline_edit_secondary_cursor,
                &mut self.inline_edit_secondary_selected,
            ),
            _ => (
                &mut self.inline_edit_buffer,
                &mut self.inline_edit_cursor,
                &mut self.inline_edit_selected,
            ),
        };

        if *selected {
            buffer.clear();
            *cursor = 0;
            *selected = false;
        } else if *cursor > 0 {
            *cursor -= 1;
            buffer.remove(*cursor);
        }

        // Live preview for scale
        if field == HeaderEditField::Scale {
            self.sync_inline_scale_preview();
        }
    }

    /// Move cursor left in inline edit
    pub fn inline_edit_left(&mut self) {
        let (cursor, selected) = match self.inline_edit_focus {
            InlineEditFocus::SecondaryInput => (
                &mut self.inline_edit_secondary_cursor,
                &mut self.inline_edit_secondary_selected,
            ),
            _ => (&mut self.inline_edit_cursor, &mut self.inline_edit_selected),
        };

        *selected = false;
        if *cursor > 0 {
            *cursor -= 1;
        }
    }

    /// Move cursor right in inline edit
    pub fn inline_edit_right(&mut self) {
        let (buffer_len, cursor, selected) = match self.inline_edit_focus {
            InlineEditFocus::SecondaryInput => (
                self.inline_edit_secondary_buffer.len(),
                &mut self.inline_edit_secondary_cursor,
                &mut self.inline_edit_secondary_selected,
            ),
            _ => (
                self.inline_edit_buffer.len(),
                &mut self.inline_edit_cursor,
                &mut self.inline_edit_selected,
            ),
        };

        *selected = false;
        if *cursor < buffer_len {
            *cursor += 1;
        }
    }

    /// Increase value in inline edit (for scroll wheel)
    pub fn inline_edit_increase(&mut self) {
        if let Some(field) = self.inline_edit {
            match field {
                HeaderEditField::Interval => {
                    if let Ok(val) = self.inline_edit_buffer.parse::<u64>() {
                        let new_val = val.saturating_add(1).min(100000);
                        self.inline_edit_buffer = new_val.to_string();
                        self.inline_edit_cursor = self.inline_edit_buffer.len();
                        self.inline_edit_selected = false;
                    }
                }
                HeaderEditField::Scale => {
                    let (buffer, cursor, selected) = match self.inline_edit_focus {
                        InlineEditFocus::SecondaryInput => (
                            &mut self.inline_edit_secondary_buffer,
                            &mut self.inline_edit_secondary_cursor,
                            &mut self.inline_edit_secondary_selected,
                        ),
                        _ => (
                            &mut self.inline_edit_buffer,
                            &mut self.inline_edit_cursor,
                            &mut self.inline_edit_selected,
                        ),
                    };
                    let new_val = Self::parse_scale_input(buffer)
                        .saturating_add(1)
                        .min(100000);
                    *buffer = new_val.to_string();
                    *cursor = buffer.len();
                    *selected = false;
                    self.sync_inline_scale_preview();
                }
                HeaderEditField::Colors => {
                    let new_scheme = self.color_scale.scheme.next();
                    self.color_scale = ColorScale::new(self.color_scale.max_rtt, new_scheme);
                    self.inline_edit_buffer = format!("{}", new_scheme);
                    self.inline_edit_selected = false;
                }
                HeaderEditField::Target => {}
            }
        }
    }

    /// Decrease value in inline edit (for scroll wheel)
    pub fn inline_edit_decrease(&mut self) {
        if let Some(field) = self.inline_edit {
            match field {
                HeaderEditField::Interval => {
                    if let Ok(val) = self.inline_edit_buffer.parse::<u64>() {
                        let new_val = val.saturating_sub(1).max(1);
                        self.inline_edit_buffer = new_val.to_string();
                        self.inline_edit_cursor = self.inline_edit_buffer.len();
                        self.inline_edit_selected = false;
                    }
                }
                HeaderEditField::Scale => {
                    let (buffer, cursor, selected) = match self.inline_edit_focus {
                        InlineEditFocus::SecondaryInput => (
                            &mut self.inline_edit_secondary_buffer,
                            &mut self.inline_edit_secondary_cursor,
                            &mut self.inline_edit_secondary_selected,
                        ),
                        _ => (
                            &mut self.inline_edit_buffer,
                            &mut self.inline_edit_cursor,
                            &mut self.inline_edit_selected,
                        ),
                    };
                    let new_val = Self::parse_scale_input(buffer).saturating_sub(1).max(1);
                    *buffer = new_val.to_string();
                    *cursor = buffer.len();
                    *selected = false;
                    self.sync_inline_scale_preview();
                }
                HeaderEditField::Colors => {
                    let new_scheme = self.color_scale.scheme.prev();
                    self.color_scale = ColorScale::new(self.color_scale.max_rtt, new_scheme);
                    self.inline_edit_buffer = format!("{}", new_scheme);
                    self.inline_edit_selected = false;
                }
                HeaderEditField::Target => {}
            }
        }
    }

    /// Cycle to next header field (Tab navigation)
    pub fn header_next_field(&mut self) {
        self.header_selected = Some(match self.header_selected {
            None => HeaderEditField::Target,
            Some(HeaderEditField::Target) => HeaderEditField::Interval,
            Some(HeaderEditField::Interval) => HeaderEditField::Scale,
            Some(HeaderEditField::Scale) => HeaderEditField::Colors,
            Some(HeaderEditField::Colors) => HeaderEditField::Target,
        });
    }

    /// Cycle to previous header field (Shift+Tab navigation)
    pub fn header_prev_field(&mut self) {
        self.header_selected = Some(match self.header_selected {
            None => HeaderEditField::Colors,
            Some(HeaderEditField::Target) => HeaderEditField::Colors,
            Some(HeaderEditField::Interval) => HeaderEditField::Target,
            Some(HeaderEditField::Scale) => HeaderEditField::Interval,
            Some(HeaderEditField::Colors) => HeaderEditField::Scale,
        });
    }

    /// Deselect header field
    pub fn header_deselect(&mut self) {
        self.header_selected = None;
    }

    /// Open inline edit for currently selected header field
    pub fn header_open_selected(&mut self) {
        if let Some(field) = self.header_selected {
            // Use position (0, 0) - will be calculated in render based on field
            self.start_inline_edit(field, 10, 1);
        }
    }

    /// Show quit confirmation dialog
    pub fn show_quit_confirm(&mut self) {
        self.quit_confirm = true;
        self.quit_confirm_no_focused = true; // Start with "No" focused (safer default)
    }

    /// Cancel quit confirmation
    pub fn cancel_quit_confirm(&mut self) {
        self.quit_confirm = false;
    }

    /// Confirm quit and exit
    pub fn confirm_quit(&mut self) {
        self.quit_confirm = false;
        self.should_quit = true;
    }
}
