pulse-tui

A local terminal-based system resource and performance monitor.

How to Install:
- exe / msi / deb / rpm: Download from the releases page (https://github.com/local76/pulse-tui/releases)
- winget: winget install local76.pulse-tui
- aur: yay -S pulse-tui-bin

## Embedding library screensaver effects (library 4.2+)

As of library 4.2, all 10 r* screensaver effects (glyphs, beams,
bounce, flame, gnats, bursts, cosmos, disco, storm,
chaos) are consolidated into the `library::role::application::scenes`
module. If your `Cargo.toml` enables the `scenes` feature, you can
embed any r* effect into this app's TUI without a separate crate:

```rust
use library::core::screensaver::Screensaver;
use library::core::TerminalCell;
use library::role::application::scenes::matrix::Matrix;

// In a Ratatui draw closure:
let mut effect = Matrix::new();
let mut grid = vec![TerminalCell::default(); cols * rows];
effect.update(std::time::Duration::from_millis(16), cols, rows);
effect.draw(&mut grid, cols, rows);
```

Available types in library 4.2:
- `scenes::matrix::Matrix`
- `scenes::beams::Beams`
- `scenes::bhop::BhopDashboard`
- `scenes::fire::FireEffect`
- `scenes::fireflies::Fireflies`
- `scenes::fireworks::Fireworks`
- `scenes::life::LifeEffect`
- `scenes::party::Party`
- `scenes::pour::Pour`
- `scenes::unstable::Unstable`

To run an effect as a standalone terminal screensaver (own raw-tty
loop, Ctrl-C to exit), use `library::screensaver_runtime::run_main`:

```rust
fn main() {
    library::screensaver_runtime::run_main(
        library::role::application::scenes::matrix::Matrix::new(),
        "glyphs",
    );
}
```

The `screensaver_runtime` module is gated on the `screensaver-runtime`
feature (default-off) — enable it in your Cargo.toml if your app needs
to host a screensaver process directly.

For the design system surface (status bar, toast, markdown viewer,
theme + accent colors, layout guard, 12 canonical TUI effects),
import the design façade:

```rust
use library::interface::tui::design::prelude::*;
```
