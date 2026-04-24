# SDRapp

> **Vibe Coded** — This application was built entirely through AI-assisted vibe coding.

A modern macOS SDR (Software Defined Radio) application built with a Rust DSP core and a native SwiftUI interface.

![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Language](https://img.shields.io/badge/language-Swift%20%2B%20Rust-orange)
![Hardware](https://img.shields.io/badge/hardware-HackRF%20%7C%20RTL--SDR-blue)

## Architecture

```
┌─────────────────────────────────────┐
│            SwiftUI (macOS)          │
│  NavigationSplitView + Metal Views  │
└────────────────┬────────────────────┘
                 │ C-ABI / FFI
┌────────────────▼────────────────────┐
│         sdrapp-core (Rust)          │
│  SoapySDR → FFT → Demod → Audio     │
└─────────────────────────────────────┘
```

- **Frontend:** SwiftUI with Metal-accelerated spectrum and waterfall displays
- **Backend:** Rust static library (`libsdrapp_core.a`) linked via C-ABI
- **Hardware:** SoapySDR abstraction layer (HackRF, RTL-SDR, and more)

## Features

- Real-time FFT spectrum display (Metal, 60 fps)
- Waterfall diagram with Viridis colormap (Metal, 30 fps)
- AM and WBFM demodulation with live audio output
- Tap-to-edit frequency input (1 kHz – 6 GHz)
- Adjustable gain control
- SoapySDR device picker

## Requirements

### Hardware
- HackRF One / PortaPack or RTL-SDR dongle

### Software
- macOS 14+ (tested on Tahoe 26.4)
- Xcode 15+
- Rust toolchain (`rustup`)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Homebrew dependencies:

```bash
brew install libusb hackrf soapyhackrf soapysdr
```

> **Note on macOS Tahoe:** Go to System Settings → Privacy & Security → Security → USB Accessories and set it to **Allow automatically** so the HackRF is recognized without a prompt.

## Building

### 1. Rust core

```bash
cd sdrapp-core
cargo build --release
cp target/release/libsdrapp_core.a ../SDRapp/Application/
```

### 2. Xcode project

```bash
cd SDRapp
xcodebuild -scheme SDRapp -configuration Debug \
  CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO
```

Or open `SDRapp/SDRapp.xcodeproj` in Xcode and press **Run**.

> The Xcode build phase runs `cargo build --release` automatically before linking.

## Project Structure

```
SDRapp/
├── sdrapp-core/          # Rust DSP library
│   └── src/
│       ├── pipeline.rs   # Receiver + DSP threads
│       ├── device.rs     # SoapySDR device abstraction
│       ├── fft.rs        # FFT processor (rustfft)
│       ├── demod.rs      # AM / WBFM demodulator
│       ├── audio.rs      # Audio output (cpal)
│       └── lib.rs        # C-ABI export
└── SDRapp/               # Xcode project
    ├── Application/
    │   └── sdrapp_core.h # cbindgen-generated C header
    └── SDRapp/
        ├── Metal/        # Spectrum.metal, Waterfall.metal
        ├── Views/        # SwiftUI views
        └── App/          # AppState, SDRCore Swift wrapper
```

## License

MIT
