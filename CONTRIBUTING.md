# Contributing to `rMonitor`

We are thrilled that you want to help improve `rMonitor`! Contributions from the community are what make open-source projects so special. Please follow these guidelines to make sure your contribution matches the style and quality standards of the project.

## 🛠️ Developer Environment Setup
To build and test `rMonitor` locally:
1. Make sure you have the standard Rust toolchain installed.
2. Clone this repository.
3. Check code formatting:
    ```bash
    cargo fmt --check
    ```
4. Run standard compiler lints:
    ```bash
    cargo clippy
    ```
5. Test the debug build:
    ```bash
    cargo run
    ```
6. Build and package the final release with the custom resource compiler script:
    ```powershell
    .\build.bat
    ```

## 📜 Pull Request Process
1. Fork the repository and create a new feature branch:
    ```bash
    git checkout -b feature/my-new-feature
    ```
2. Write clean code and keep your changes focused.
3. Make sure all compile checks and lints pass.
4. Document any new features in the [README.md](README.md) or corresponding help manuals.
5. Open a Pull Request detailing the purpose of your change and any design decisions you made.

## 🎨 TUI Design Principles
If you are modifying the user interface, please keep in mind:
*   **Aesthetics:** We use high-contrast HSL/RGB tailored color themes. Do not use plain primaries (e.g., pure blue, pure red).
*   **Balance:** Maintain 4-line layouts in the top statistics panels to keep the dashboard balanced.
*   **Compact Core Grid:** Keep core layouts wrapped so they support very high core counts (up to 64+ logical cores) without breaking borders.
