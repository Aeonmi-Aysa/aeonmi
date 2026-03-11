//! GDF — Glyph Derivation Function
//! Maps a 64-byte seed to visual and acoustic parameters: OKLCH color, Hz frequency, geometry.

/// OKLCH perceptual color (no sRGB clamping needed at rendering time).
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphColor {
    /// Perceived lightness [0.0, 1.0]
    pub lightness: f64,
    /// Chroma (colorfulness) [0.0, 0.4]
    pub chroma: f64,
    /// Hue angle in degrees [0.0, 360.0)
    pub hue: f64,
}

impl GlyphColor {
    /// Convert to approximate sRGB for ANSI terminal rendering.
    /// This is a fast approximation — not perceptually perfect but good enough for terminals.
    pub fn to_srgb(&self) -> (u8, u8, u8) {
        // OKLCH → OKLab
        let hue_rad = self.hue.to_radians();
        let a = self.chroma * hue_rad.cos();
        let b_val = self.chroma * hue_rad.sin();
        let l = self.lightness;

        // OKLab → linear sRGB (approximate)
        let l_ = l + 0.3963377774 * a + 0.2158037573 * b_val;
        let m_ = l - 0.1055613458 * a - 0.0638541728 * b_val;
        let s_ = l - 0.0894841775 * a - 1.2914855480 * b_val;

        let l3 = l_ * l_ * l_;
        let m3 = m_ * m_ * m_;
        let s3 = s_ * s_ * s_;

        let r_lin = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
        let g_lin = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
        let b_lin = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

        // Clamp + gamma (sRGB transfer function)
        let srgb = |v: f64| -> u8 {
            let v = v.clamp(0.0, 1.0);
            let v = if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (v * 255.0).round() as u8
        };

        (srgb(r_lin), srgb(g_lin), srgb(b_lin))
    }
}

/// Glyph visual and acoustic parameters derived from a seed.
#[derive(Debug, Clone)]
pub struct GlyphParams {
    /// OKLCH color
    pub color: GlyphColor,
    /// Frequency in Hz (432–528 range for harmony; 111 Hz = distortion)
    pub frequency_hz: f64,
    /// Rotation angle 0–360° for glyph geometry
    pub rotation_deg: f64,
    /// Pulse rate in BPM (60–120 range for harmony)
    pub pulse_bpm: f64,
    /// Whether this glyph is in distortion/anomaly state
    pub distorted: bool,
}

impl GlyphParams {
    /// Derive GlyphParams from a 64-byte HKDF seed.
    pub fn from_seed(seed: &[u8; 64]) -> Self {
        // Use different byte ranges for different parameters to avoid correlation.
        let lightness = 0.4 + (seed[0] as f64 / 255.0) * 0.5; // 0.4–0.9
        let chroma = 0.1 + (seed[1] as f64 / 255.0) * 0.27;   // 0.10–0.37
        let hue = (u16::from_le_bytes([seed[2], seed[3]]) as f64 / 65535.0) * 360.0;

        // Hz: map bytes 4–5 to 432–528 Hz (harmonic band from Tesla principles)
        let hz_raw = u16::from_le_bytes([seed[4], seed[5]]) as f64 / 65535.0;
        let frequency_hz = 432.0 + hz_raw * 96.0;

        // Rotation: bytes 6–7
        let rotation_deg = (u16::from_le_bytes([seed[6], seed[7]]) as f64 / 65535.0) * 360.0;

        // Pulse BPM: bytes 8–9 → 60–120
        let pulse_bpm = 60.0 + (seed[8] as f64 / 255.0) * 60.0;

        Self {
            color: GlyphColor { lightness, chroma, hue },
            frequency_hz,
            rotation_deg,
            pulse_bpm,
            distorted: false,
        }
    }

    /// Apply anomaly distortion: hue flip, frequency → 111 Hz (dissonance), lightness drop.
    pub fn distort(&mut self) {
        self.color.hue = (self.color.hue + 180.0) % 360.0;
        self.color.lightness = (self.color.lightness - 0.2).max(0.1);
        self.color.chroma = (self.color.chroma - 0.1).max(0.0);
        self.frequency_hz = 111.0; // dissonant frequency
        self.distorted = true;
    }

    /// Render the glyph as ANSI colored terminal art. Returns a String with escape codes.
    pub fn render_terminal(&self) -> String {
        let (r, g, b) = self.color.to_srgb();
        let (r2, g2, b2) = {
            // Inner glow: slightly brighter
            let inner = GlyphColor {
                lightness: (self.color.lightness + 0.15).min(1.0),
                chroma: self.color.chroma,
                hue: self.color.hue,
            };
            inner.to_srgb()
        };

        let reset = "\x1b[0m";
        let outer_color = format!("\x1b[38;2;{};{};{}m", r, g, b);
        let inner_color = format!("\x1b[38;2;{};{};{}m", r2, g2, b2);

        let status_line = if self.distorted {
            format!("  \x1b[38;2;255;50;50m⚠ ANOMALY DETECTED — GLYPH DISTORTED{}", reset)
        } else {
            format!("  {}✓ GLYPH HARMONIZED{}", outer_color, reset)
        };

        let freq_str = if self.distorted {
            format!("{:.0}Hz [DISSONANT]", self.frequency_hz)
        } else {
            format!("{:.0}Hz [{:.1}°]", self.frequency_hz, self.color.hue)
        };

        format!(
            "{outer}
    {outer}  ╭─────────────────╮{reset}
    {outer}  │  {inner}     △     {outer}  │{reset}
    {outer}  │  {inner}    ∞∞∞    {outer}  │{reset}
    {outer}  │  {inner}     ◎     {outer}  │{reset}
    {outer}  ╰─────────────────╯{reset}
    {outer}  ◈ AEONMI SHARD   ◈{reset}
    {status}
    {outer}  {freq}{reset}
",
            outer = outer_color,
            inner = inner_color,
            status = status_line,
            freq = freq_str,
        )
    }

    /// Return a one-line status suitable for CLI output.
    pub fn status_line(&self) -> String {
        let state = if self.distorted { "⚠ DISTORTED" } else { "✓ HARMONIZED" };
        format!(
            "[Glyph] {} | hue={:.1}° | {:.0}Hz | L={:.2} C={:.2}",
            state,
            self.color.hue,
            self.frequency_hz,
            self.color.lightness,
            self.color.chroma,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; 64] {
        let mut s = [0u8; 64];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i * 7 + 13) as u8;
        }
        s
    }

    #[test]
    fn test_params_in_valid_ranges() {
        let seed = test_seed();
        let p = GlyphParams::from_seed(&seed);
        assert!((0.4..=0.9).contains(&p.color.lightness), "lightness out of range: {}", p.color.lightness);
        assert!((0.1..=0.37).contains(&p.color.chroma), "chroma out of range: {}", p.color.chroma);
        assert!((0.0..360.0).contains(&p.color.hue), "hue out of range: {}", p.color.hue);
        assert!((432.0..=528.0).contains(&p.frequency_hz), "Hz out of range: {}", p.frequency_hz);
        assert!(!p.distorted);
    }

    #[test]
    fn test_distortion_changes_values() {
        let seed = test_seed();
        let mut p = GlyphParams::from_seed(&seed);
        let orig_hue = p.color.hue;
        let orig_hz = p.frequency_hz;
        p.distort();
        assert!(p.distorted);
        assert_ne!(p.color.hue, orig_hue, "Distortion should change hue");
        assert_eq!(p.frequency_hz, 111.0, "Distortion should set Hz to 111");
    }

    #[test]
    fn test_render_returns_nonempty() {
        let seed = test_seed();
        let p = GlyphParams::from_seed(&seed);
        let rendered = p.render_terminal();
        assert!(!rendered.is_empty());
        assert!(rendered.contains("AEONMI SHARD"));
    }

    #[test]
    fn test_srgb_clamps_to_valid_range() {
        let color = GlyphColor { lightness: 0.7, chroma: 0.25, hue: 240.0 };
        let (r, g, b) = color.to_srgb();
        // All must be 0–255 (enforced by u8 type)
        let _ = (r, g, b);
    }
}
