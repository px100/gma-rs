# gba-rs

`gba-rs` is a single-source Rust Game Boy Advance emulator. Core emulation, SDL2 frontend, save handling, harness mode, and tests live in `src/main.rs`

## Requirements

- Rust 1.96.0, pinned by `rust-toolchain.toml`
- CMake
- macOS: Xcode Command Line Tools

SDL2 is built from the bundled crate source and statically linked; no system SDL2 package is required.

## Install

```sh
rustup toolchain install 1.96.0
cargo build --release
```

The optimized binary is `target/release/gba-rs`.

## Run

```sh
cargo run --release -- path/to/game.gba
```

Options:

```sh
--bios path/to/gba_bios.bin
--skip-bios
--scale 4
--no-audio
--harness
--test-pattern
--test-solid-red
```

Example:

```sh
cargo run --release -- path/to/game.gba --bios path/to/gba_bios.bin --scale 4
```

Controls:

```text
D-pad: Arrow keys or W/A/S/D
A/B: Z/X or J/K
Start: Enter or Space
Select: Tab, Backspace, or Shift
L/R: Q/E
Save state: ]
Load state: [
```

Save files are loaded from and written beside the ROM.

## Test

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT. See `LICENSE`.
