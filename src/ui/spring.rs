//! Damped Harmonic Oscillator (spring physics) for animated TUI elements.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub value: f64,
    pub velocity: f64,
    pub target: f64,
    pub tension: f64,
    pub damping: f64,
}

impl Spring {
    pub fn new(tension: f64, damping: f64) -> Self {
        Self {
            value: 0.0,
            velocity: 0.0,
            target: 0.0,
            tension,
            damping,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let force = self.tension * (self.target - self.value) - self.damping * self.velocity;
        self.velocity += force * dt;
        self.value += self.velocity * dt;
    }
}
