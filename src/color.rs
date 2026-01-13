use clap::ValueEnum;
use ratatui::style::Color;

/// Available color schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorScheme {
    /// Classic green-yellow-red gradient
    Classic,
    /// Dark theme - low pings are nearly invisible, spikes stand out
    #[default]
    Dark,
    /// Ocean blue theme - dark blue to cyan to white
    Ocean,
    /// Fire theme - black to red to yellow to white
    Fire,
    /// Neon theme - dark purple to bright neon pink/cyan
    Neon,
    /// Grayscale - black to white
    Grayscale,
    /// Matrix theme - dark to bright green
    Matrix,
    /// Plasma theme - purple to pink to white
    Plasma,
    /// Ice theme - dark blue to light cyan to white
    Ice,
    /// Thermal camera style - blue to cyan to green to yellow to red
    Thermal,
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorScheme::Classic => write!(f, "Classic"),
            ColorScheme::Dark => write!(f, "Dark"),
            ColorScheme::Ocean => write!(f, "Ocean"),
            ColorScheme::Fire => write!(f, "Fire"),
            ColorScheme::Neon => write!(f, "Neon"),
            ColorScheme::Grayscale => write!(f, "Grayscale"),
            ColorScheme::Matrix => write!(f, "Matrix"),
            ColorScheme::Plasma => write!(f, "Plasma"),
            ColorScheme::Ice => write!(f, "Ice"),
            ColorScheme::Thermal => write!(f, "Thermal"),
        }
    }
}

impl ColorScheme {
    /// Get the next color scheme in the cycle
    pub fn next(self) -> Self {
        match self {
            ColorScheme::Classic => ColorScheme::Dark,
            ColorScheme::Dark => ColorScheme::Ocean,
            ColorScheme::Ocean => ColorScheme::Fire,
            ColorScheme::Fire => ColorScheme::Neon,
            ColorScheme::Neon => ColorScheme::Grayscale,
            ColorScheme::Grayscale => ColorScheme::Matrix,
            ColorScheme::Matrix => ColorScheme::Plasma,
            ColorScheme::Plasma => ColorScheme::Ice,
            ColorScheme::Ice => ColorScheme::Thermal,
            ColorScheme::Thermal => ColorScheme::Classic,
        }
    }

    /// Get the previous color scheme in the cycle
    pub fn prev(self) -> Self {
        match self {
            ColorScheme::Classic => ColorScheme::Thermal,
            ColorScheme::Dark => ColorScheme::Classic,
            ColorScheme::Ocean => ColorScheme::Dark,
            ColorScheme::Fire => ColorScheme::Ocean,
            ColorScheme::Neon => ColorScheme::Fire,
            ColorScheme::Grayscale => ColorScheme::Neon,
            ColorScheme::Matrix => ColorScheme::Grayscale,
            ColorScheme::Plasma => ColorScheme::Matrix,
            ColorScheme::Ice => ColorScheme::Plasma,
            ColorScheme::Thermal => ColorScheme::Ice,
        }
    }
}

/// Linear interpolation between two values
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Interpolate between two RGB colors
fn lerp_rgb(c1: (u8, u8, u8), c2: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        lerp(c1.0 as f64, c2.0 as f64, t) as u8,
        lerp(c1.1 as f64, c2.1 as f64, t) as u8,
        lerp(c1.2 as f64, c2.2 as f64, t) as u8,
    )
}

/// Interpolate through a list of color stops
fn gradient(stops: &[(f64, (u8, u8, u8))], t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);

    // Find the two stops to interpolate between
    for i in 0..stops.len() - 1 {
        let (t1, c1) = stops[i];
        let (t2, c2) = stops[i + 1];

        if t >= t1 && t <= t2 {
            let local_t = (t - t1) / (t2 - t1);
            return lerp_rgb(c1, c2, local_t);
        }
    }

    // Return last color if beyond range
    stops.last().map(|(_, c)| *c).unwrap_or((255, 255, 255))
}

/// Color gradient for RTT visualization using true RGB colors
pub struct ColorScale {
    /// RTT value (ms) that is considered "bad"
    pub max_rtt: u64,
    /// Color scheme to use
    pub scheme: ColorScheme,
}

impl ColorScale {
    pub fn new(max_rtt: u64, scheme: ColorScheme) -> Self {
        Self { max_rtt, scheme }
    }

    /// Get the color stops for the current scheme
    /// Each stop is (position 0.0-1.0, RGB color)
    fn get_stops(&self) -> Vec<(f64, (u8, u8, u8))> {
        match self.scheme {
            ColorScheme::Classic => vec![
                (0.0, (0, 255, 0)),    // Bright green
                (0.25, (128, 255, 0)), // Yellow-green
                (0.5, (255, 255, 0)),  // Yellow
                (0.75, (255, 128, 0)), // Orange
                (1.0, (255, 0, 0)),    // Red
            ],
            ColorScheme::Dark => vec![
                (0.0, (15, 10, 25)),     // Almost black/dark purple
                (0.1, (25, 15, 40)),     // Very dark purple
                (0.3, (40, 30, 70)),     // Dark purple
                (0.5, (60, 80, 120)),    // Muted blue
                (0.7, (80, 180, 220)),   // Cyan
                (0.85, (120, 220, 255)), // Bright cyan
                (1.0, (255, 255, 255)),  // White
            ],
            ColorScheme::Ocean => vec![
                (0.0, (0, 20, 40)),      // Very dark blue
                (0.3, (0, 60, 120)),     // Dark blue
                (0.5, (0, 120, 180)),    // Blue
                (0.7, (0, 180, 220)),    // Cyan
                (0.85, (100, 220, 255)), // Light cyan
                (1.0, (255, 255, 255)),  // White
            ],
            ColorScheme::Fire => vec![
                (0.0, (10, 0, 0)),      // Almost black
                (0.2, (60, 0, 0)),      // Very dark red
                (0.4, (180, 30, 0)),    // Dark red
                (0.6, (255, 100, 0)),   // Orange
                (0.8, (255, 200, 0)),   // Yellow-orange
                (1.0, (255, 255, 200)), // Pale yellow/white
            ],
            ColorScheme::Neon => vec![
                (0.0, (10, 0, 20)),     // Almost black
                (0.2, (40, 0, 60)),     // Dark purple
                (0.4, (120, 0, 180)),   // Purple
                (0.6, (200, 0, 255)),   // Bright purple
                (0.8, (255, 0, 200)),   // Pink
                (1.0, (255, 100, 255)), // Bright pink
            ],
            ColorScheme::Grayscale => vec![
                (0.0, (20, 20, 20)),    // Almost black
                (0.5, (128, 128, 128)), // Gray
                (1.0, (255, 255, 255)), // White
            ],
            ColorScheme::Matrix => vec![
                (0.0, (0, 15, 0)),       // Almost black
                (0.3, (0, 60, 0)),       // Dark green
                (0.5, (0, 120, 0)),      // Green
                (0.7, (0, 200, 0)),      // Bright green
                (0.85, (100, 255, 100)), // Light green
                (1.0, (200, 255, 200)),  // Pale green
            ],
            ColorScheme::Plasma => vec![
                (0.0, (10, 0, 20)),     // Almost black
                (0.2, (60, 0, 100)),    // Dark purple
                (0.4, (140, 0, 160)),   // Purple
                (0.6, (200, 50, 180)),  // Magenta
                (0.8, (255, 120, 200)), // Pink
                (1.0, (255, 220, 255)), // Pale pink
            ],
            ColorScheme::Ice => vec![
                (0.0, (0, 10, 30)),      // Almost black/dark blue
                (0.25, (20, 40, 80)),    // Dark blue
                (0.5, (60, 100, 160)),   // Blue
                (0.7, (100, 180, 220)),  // Light blue
                (0.85, (180, 230, 255)), // Ice blue
                (1.0, (240, 250, 255)),  // Almost white
            ],
            ColorScheme::Thermal => vec![
                (0.0, (0, 0, 40)),     // Dark blue
                (0.2, (0, 80, 160)),   // Blue
                (0.35, (0, 180, 180)), // Cyan
                (0.5, (0, 200, 80)),   // Green
                (0.65, (180, 220, 0)), // Yellow-green
                (0.8, (255, 180, 0)),  // Orange
                (1.0, (255, 60, 60)),  // Red
            ],
        }
    }

    /// Get color for a given RTT value using smooth RGB gradient
    pub fn color_for_rtt(&self, rtt_ms: Option<u64>) -> Color {
        match rtt_ms {
            None => Color::Indexed(240), // Gray for timeout
            Some(rtt) => {
                // Calculate percentage of max RTT (capped at 100%)
                let ratio = (rtt as f64 / self.max_rtt as f64).min(1.0);
                let (r, g, b) = gradient(&self.get_stops(), ratio);
                Color::Rgb(r, g, b)
            }
        }
    }

    /// Get color for a given RTT value (f64 version for sub-ms precision)
    pub fn color_for_rtt_f64(&self, rtt_ms: Option<f64>) -> Color {
        match rtt_ms {
            None => Color::Indexed(240),
            Some(rtt) => {
                let ratio = (rtt / self.max_rtt as f64).min(1.0);
                let (r, g, b) = gradient(&self.get_stops(), ratio);
                Color::Rgb(r, g, b)
            }
        }
    }

    /// Get legend entries as (color, label) pairs
    pub fn legend_entries(&self) -> Vec<(Color, String)> {
        // Generate 10 evenly spaced legend entries
        let num_entries = 10;
        let mut entries = Vec::new();

        for i in 0..num_entries {
            let ratio = i as f64 / (num_entries - 1) as f64;
            let rtt = (ratio * self.max_rtt as f64) as u64;
            let color = self.color_for_rtt(Some(rtt));

            if i == num_entries - 1 {
                entries.push((color, format!("{}ms+", rtt)));
            } else {
                let next_rtt =
                    ((i + 1) as f64 / (num_entries - 1) as f64 * self.max_rtt as f64) as u64;
                entries.push((color, format!("{}-{}ms", rtt, next_rtt)));
            }
        }

        // Add timeout entry
        entries.push((Color::Indexed(240), "Timeout".to_string()));

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scale_rgb() {
        let scale = ColorScale::new(200, ColorScheme::Classic);

        // Test that we get RGB colors
        let color = scale.color_for_rtt(Some(100));
        assert!(matches!(color, Color::Rgb(_, _, _)));

        // Timeout should still be indexed
        assert!(matches!(scale.color_for_rtt(None), Color::Indexed(240)));
    }

    #[test]
    fn test_gradient_interpolation() {
        let stops = vec![(0.0, (0u8, 0u8, 0u8)), (1.0, (255u8, 255u8, 255u8))];

        // Midpoint should be gray
        let (r, g, b) = gradient(&stops, 0.5);
        assert!(r > 120 && r < 135); // ~127
        assert!(g > 120 && g < 135);
        assert!(b > 120 && b < 135);
    }

    #[test]
    fn test_all_schemes() {
        // Ensure all schemes work without panicking
        for scheme in [
            ColorScheme::Classic,
            ColorScheme::Dark,
            ColorScheme::Ocean,
            ColorScheme::Fire,
            ColorScheme::Neon,
            ColorScheme::Grayscale,
            ColorScheme::Matrix,
            ColorScheme::Plasma,
            ColorScheme::Ice,
            ColorScheme::Thermal,
        ] {
            let scale = ColorScale::new(100, scheme);
            let _ = scale.color_for_rtt(Some(50));
            let _ = scale.legend_entries();
        }
    }
}
