# GEMINI.md - baloogah

## Project Overview

**baloogah** is a Rust CLI tool designed to automatically pull the latest versions of Docker images that are currently used by running containers. It interacts directly with the Docker Engine API via the local Unix socket to identify active images and performs concurrent updates.

### Main Technologies
- **Rust (Edition 2021)**: The core programming language.
- **Tokio**: Used as the asynchronous runtime for concurrent image pulling.
- **docker-api**: A Rust library for interacting with the Docker Engine API.
- **Anyhow**: For flexible and ergonomic error handling.
- **Cross**: For cross-compilation (specifically targeting ARM64 Linux).

### Architecture
- `src/main.rs`: Entry point that orchestrates the workflow:
    1. Connects to the Docker daemon.
    2. Identifies running containers and their images.
    3. Initializes the TUI and background pull tasks.
    4. Manages the main event loop (UI rendering and channel processing).
- `src/docker.rs`: Logic for interacting with the Docker API, image listing, and piping pull updates over an `mpsc` channel.
- `src/app.rs`: Application state management, tracking overall progress, individual image statuses, and errors.
- `src/tui.rs`: UI rendering logic using `ratatui`, defining the layout and widgets for the progress display and error pane.

## Building and Running

### Prerequisites
- Docker must be running and accessible via `/var/run/docker.sock`.
- Rust (Cargo) installed.

### Key Commands
- **Build**: `cargo build`
- **Run**: `cargo run` (Note: May require `sudo` or membership in the `docker` group to access the socket)
- **Test**: `cargo test`
- **Cross-compile (ARM64)**: `cross build --target aarch64-unknown-linux-gnu` (as defined in `Cross.toml`)

## Development Conventions

### Coding Style
- Follows idiomatic Rust conventions (use `cargo fmt` for formatting).
- Employs `tokio` for all asynchronous operations.
- Uses `Arc` for sharing the Docker client across multiple tasks.
- Error handling is centralized using `anyhow::Result`.

### Testing
- Basic unit tests can be found in `src/docker.rs`.
- Run tests with `cargo test`.
- **Note**: Always run tests after making code changes to verify they work and prevent regressions.

### Project Structure
- `src/`: Contains all source code.
    - `main.rs`: Main entry point.
    - `docker.rs`: Docker Engine API integration.
    - `app.rs`: TUI state management.
    - `tui.rs`: TUI rendering logic.
- `Cargo.toml`: Project manifest and dependency definitions.
- `Cross.toml`: Configuration for cross-compilation with `cross`.
