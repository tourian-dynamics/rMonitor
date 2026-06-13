use std::time::{Duration, Instant};
use ratatui::{backend::TestBackend, Terminal};
use crate::app::App;
use crate::config::AppConfig;
use crate::ui::draw;

#[test]
fn test_ui_rendering_perf_budget() {
    let config = AppConfig::default();
    let mut app = App::new(config);
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create test terminal");
    
    // Warmup
    terminal.draw(|f| draw(f, &mut app)).unwrap();
    
    // Benchmark 100 frames
    const FRAMES: usize = 100;
    let start = Instant::now();
    for _ in 0..FRAMES {
        terminal.draw(|f| draw(f, &mut app)).unwrap();
    }
    let elapsed = start.elapsed();
    
    let budget = Duration::from_millis(10000);
    assert!(
        elapsed < budget,
        "100 frames took {:?}, exceeding budget of {:?}",
        elapsed,
        budget
    );
    println!("TUI Render Loop Performance: {} frames in {:?}", FRAMES, elapsed);
}
