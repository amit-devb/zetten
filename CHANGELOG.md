# Changelog

All notable changes to the Zetten task runner will be documented in this file.

---
## [1.0.10] – 2026-01-06

### Changed
- Switched Python distribution to **maturin-based native wheels**
- Rust CLI binary is now bundled directly inside PyPI wheels
- Installation via `pip` and `pipx` is now fully native and deterministic
- Release pipeline split into dedicated workflows per distribution channel

### Added
- Native PyPI wheels for:
  - Linux
  - macOS (Intel & Apple Silicon)
  - Windows
- `pipx` as a first-class, recommended installation method
- Binary distribution via `cargo-dist` with GitHub Releases
- Dedicated crates.io publishing workflow for Rust users
- CI gating and dry-run protection for crates.io releases

### Removed
- Removed runtime Python installer that downloaded binaries from GitHub Releases
- Removed install-time platform detection and checksum verification
- Removed dependency on GitHub Releases for Python installations

### Fixed
- Fixed Windows installation failures caused by GitHub release redirects
- Eliminated checksum mismatch errors during installation
- Removed all install-time network dependencies for Python users

### Notes
- This release changes **how Zetten is distributed**, but **does not change CLI behavior**
- Existing users can upgrade normally via `pip` or `pipx`
- Recommended installation method going forward:
  ```bash
  pipx install zetten
  ```

---

## [1.0.9] – 2026-01-04
### Fixed
- Fixed Windows installation failure caused by GitHub `latest` release redirects
- Installer now downloads version-pinned GitHub release assets
- Improved checksum validation robustness across all platforms

## [1.0.8] - 2026-01-04
### 🚀 Features
- **Variable Engine**: Implemented 3-tier resolution (CLI > TOML > Environment).
- **Fallback Syntax**: Support for `${VAR:-default}` for optional configurations.
- **Worker Pool**: High-performance parallel execution using Kahn's algorithm.
- **Process Registry**: Global signal handling to prevent zombie subprocesses.
- **Critical Path Analysis**: Post-run analytics to identify task bottlenecks.

### 🛠 UI/UX
- **TUI Selector**: Interactive fuzzy-finder for task selection via `inquire`.
- **Precision Watch**: Smart watcher that re-runs only tasks affected by file changes.
- **Dry Run**: Added `--dry-run` flag to preview command interpolation.

---

## [1.0.5] - 2025-11-15
### 🚀 Features
- **Pyproject.toml Integration**: Native support for reading tasks from Python project files.
- **Tagging System**: Group tasks by logical categories (e.g., `ci`, `lint`).
- **Dependency Graph**: Added the `zetten graph` command to visualize task relationships.

### ⚙️ Improvements
- **Concurrency**: Introduced a basic thread pool for non-dependent task execution.
- **Logging**: Switched to a structured logging system for cleaner terminal output.

---

## [1.0.0] - 2025-09-01
### 🚀 Features
- **Core Engine**: Initial release of the Rust-based task runner.
- **Basic Watcher**: Simple file monitoring for task re-execution.
- **Init Command**: `zetten init` for interactive project scaffolding.
- **Cross-Platform**: Initial support for Windows (powershell) and Unix (sh) shells.

---

## [0.1.0] - 2025-06-15
### 🚀 Features
- **Prototype**: Proof-of-concept release featuring basic shell command execution.
- **Config**: Initial support for `zetten.toml` configuration format.