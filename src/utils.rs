//! General math and utility helpers.

/// Calculate percentage from two unsigned integers.
/// Returns 0.0 if total is 0 to avoid division by zero.
pub fn percentage(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f32 / total as f32) * 100.0
    }
}

/// Linear interpolation between two values.
/// Clamps the factor to [0.0, 1.0] for safety.
pub fn lerp(a: f32, b: f32, factor: f32) -> f32 {
    let clamped_factor = factor.clamp(0.0, 1.0);
    a + (b - a) * clamped_factor
}

/// Convert HSL color coordinates to RGB.
/// `h` is in [0.0, 360.0], `s` and `l` are in [0.0, 1.0].
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Simple pseudo-random noise based on time and index.
/// Uses a combination of sine waves with irrational multipliers to create
/// smooth, deterministic noise that varies over time.
pub fn smooth_noise(elapsed_secs: f64, index: usize, amplitude: f64, frequency_base: f64) -> f64 {
    const IRRATIONAL_1: f64 = 1.618033988749895; // Golden ratio
    const IRRATIONAL_2: f64 = std::f64::consts::E;
    
    let freq1 = frequency_base * (1.0 + index as f64 * 0.3);
    let freq2 = frequency_base * (1.0 + index as f64 * 0.5);
    
    let noise = (elapsed_secs * freq1 * IRRATIONAL_1).sin() * 0.6
              + (elapsed_secs * freq2 * IRRATIONAL_2).cos() * 0.4;
    
    noise * amplitude
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;

