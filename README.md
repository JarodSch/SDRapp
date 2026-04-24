# SDRapp

> **Vibe Coded** — This application was built entirely through AI-assisted vibe coding.

A modern macOS SDR (Software Defined Radio) application with a Rust DSP core and a native SwiftUI/Metal interface.

![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Language](https://img.shields.io/badge/language-Swift%20%2B%20Rust-orange)
![Hardware](https://img.shields.io/badge/hardware-HackRF%20%7C%20RTL--SDR-blue)

---

## Features

- Real-time FFT spectrum display (Metal, 60 fps)
- Waterfall diagram with Viridis colormap (Metal, 30 fps)
- AM and WBFM demodulation with live audio output
- **Click-to-tune** — click anywhere in spectrum or waterfall to jump to that frequency
- **Scroll-to-pan** — trackpad / mouse wheel shifts center frequency
- Hover cursor shows frequency label
- Tap-to-edit frequency input (1 kHz – 6 GHz)
- Per-element gain control (LNA / VGA / AMP — queried live from SoapySDR)
- SoapySDR device picker with async refresh
- Live frequency and gain tuning without stop/restart
- Military Amber dark theme with Metal-rendered visuals
- Native macOS window chrome integration (hidden title bar)

---

## Architecture

```
┌─────────────────────────────────────────────┐
│              SwiftUI (macOS)                │
│  Sidebar  │  Spectrum (Metal)  │  Waterfall  │
└────────────────────┬────────────────────────┘
                     │ C-ABI / FFI (cbindgen header)
┌────────────────────▼────────────────────────┐
│             sdrapp-core (Rust)              │
│                                             │
│  SoapySDR → Ring Buffer → FFT → Demod → Audio
│               (receiver     (DSP thread)    │
│                thread)                      │
└─────────────────────────────────────────────┘
```

**Threading model:**
- **Receiver thread** — SoapySDR → `HeapRb` ring buffer (lock-free)
- **DSP thread** — ring buffer → FFT → demodulation → audio output (cpal)
- **Live tuning** — `soapysdr::Device` shared via `Arc<Mutex<>>` so frequency and gain changes reach hardware immediately without restarting

---

## Requirements

### Hardware
- HackRF One / PortaPack H4M Mayhem, or any RTL-SDR dongle

### macOS Setup

1. **Xcode** 15 or later
2. **Rust toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. **Homebrew dependencies**
   ```bash
   brew install libusb hackrf soapyhackrf soapysdr
   ```
   > `libusb` is the critical userspace USB layer. Without it SoapySDR cannot see any SDR hardware on macOS.

4. **USB Accessories permission** (macOS Tahoe 26.x)
   System Settings → Privacy & Security → Security → USB Accessories → **Allow automatically**

5. **HackRF PortaPack users:** Power on, then select **HackRF USB Mode** from the PortaPack menu before connecting.

---

## Building

### 1. Build the Rust core

```bash
cd sdrapp-core
cargo build           # debug
# or
cargo build --release # release — then copy to SDRapp/Application/
```

For a release build, copy the library manually:
```bash
cp target/release/libsdrapp_core.a ../SDRapp/Application/
```

> The Xcode project references the library from `SDRapp/Application/`. Debug builds are picked up automatically by the Xcode build phase script.

### 2. Build and run in Xcode

```bash
cd SDRapp
xcodebuild -scheme SDRapp -configuration Debug \
  CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO
```

Or open `SDRapp/SDRapp.xcodeproj` in Xcode and press **Run** (⌘R).

> **SourceKit false errors** — Xcode's SourceKit may show "Cannot find 'Theme' in scope" etc. These are false positives caused by `PBXFileSystemSynchronizedRootGroup`. `xcodebuild` compiles correctly; ignore them.

---

## Project Structure

```
SDRapp/
├── sdrapp-core/               # Rust DSP library (static)
│   └── src/
│       ├── lib.rs             # C-ABI exports (cbindgen → sdrapp_core.h)
│       ├── pipeline.rs        # Receiver thread + DSP thread + live tuning
│       ├── device.rs          # SoapySDR device abstraction + gain elements
│       ├── fft.rs             # FFT processor (rustfft, 1024-point)
│       ├── demod.rs           # AM / WBFM demodulator
│       └── audio.rs           # Audio output (cpal / CoreAudio)
│
└── SDRapp/                    # Xcode project
    ├── Application/
    │   ├── sdrapp_core.h      # Auto-generated C header (cbindgen)
    │   └── libsdrapp_core.a   # Compiled Rust static library (gitignored)
    └── SDRapp/
        ├── SDRappApp.swift    # App entry point, .hiddenTitleBar
        ├── ContentView.swift  # HSplitView: Sidebar | Spectrum+Waterfall
        ├── Theme.swift        # Military Amber color & font constants
        ├── SDRCore.swift      # Swift wrapper around C-ABI + GainElement struct
        ├── WindowConfigurator.swift  # NSViewRepresentable for window chrome
        ├── App/
        │   └── AppState.swift # @Observable state, device/frequency/gain logic
        ├── Views/
        │   ├── Sidebar/
        │   │   ├── SidebarView.swift       # Root sidebar layout
        │   │   ├── DevicePickerView.swift  # Device picker + async refresh
        │   │   ├── ModePickerView.swift    # AM / WBFM toggle
        │   │   └── GainControlView.swift   # Per-element gain sliders
        │   └── Spectrum/
        │       ├── FrequencyBarView.swift      # Frequency display + tap-to-edit
        │       ├── SpectrumContainerView.swift # Layout + click/scroll interaction
        │       ├── SpectrumRenderer.swift      # Metal FFT line + fill renderer
        │       ├── WaterfallMetalView.swift    # NSViewRepresentable for waterfall
        │       └── WaterfallRenderer.swift     # Metal waterfall with Viridis colormap
        └── Metal/
            ├── Spectrum.metal   # FFT line + gradient fill shader (amber)
            └── Waterfall.metal  # Scrolling waterfall shader (Viridis colormap)
```

---

## Key Implementation Notes

### FFT & Frequency Mapping
- FFT size: **1024 bins**
- Sample rate: **2,048,000 Hz** (2.048 MHz visible bandwidth)
- Left edge = `centerFrequency − 1,024,000 Hz`
- Right edge = `centerFrequency + 1,024,000 Hz`
- Click at normalized position `x` → `newFreq = center − 1,024,000 + x × 2,048,000`

### Live Tuning
- `soapysdr::Device` is stored in `Arc<Mutex<Option<Device>>>` (`live_device` in `pipeline.rs`)
- `set_frequency()` and `set_gain_element()` acquire the lock and call hardware directly
- No stop/restart needed

### Per-Element Gain
- `SdrDevice::list_gain_elements()` calls `device.list_gains()` + `device.gain_element_range()` + `device.gain_element()` per element
- HackRF elements: **LNA** (0–40 dB, 8 dB steps), **VGA** (0–62 dB, 2 dB steps), **AMP** (0/14 dB)
- Falls back to single Overall slider when no device is selected

### macOS Button Hit Areas
- All sidebar buttons use `Button { } label: { ... .contentShape(Rectangle()) }` pattern
- `.contentShape` must be inside the label ViewBuilder, not on the Button itself — on macOS with `.buttonStyle(.plain)` the hit area is determined by the label content, not outer modifiers

---

## License

MIT
