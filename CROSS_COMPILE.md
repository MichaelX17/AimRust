# Cross-Compiling to Windows (x86_64-pc-windows-msvc)

Instructions for cross-compiling AimRust from Linux to Windows using `cargo-xwin`.

## Prerequisites

1. **Install the Rust Windows MSVC Target:**
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```

2. **Install `cargo-xwin`:**
   ```bash
   cargo install cargo-xwin
   ```

## Building

Run the provided build script:
```bash
chmod +x build_windows.sh && ./build_windows.sh
```

Alternatively, run `cargo-xwin` directly:
```bash
cargo xwin build --target x86_64-pc-windows-msvc --release
```

## Binary Output Location

Upon completion, the compiled Windows executable will be located at:
```text
target/x86_64-pc-windows-msvc/release/AimRust.exe
```

> **Note:** Remember to package the `assets/` directory alongside `AimRust.exe` when distributing the build.
