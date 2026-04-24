# SDRapp Phase 1 — Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eine lauffähige macOS-SDR-App die FM-Radio empfangen, Spektrum und Wasserfall anzeigen kann — Rust für DSP, SwiftUI für UI.

**Architecture:** Rust DSP Core (SoapySDR → Ring Buffer → FFT → Demodulator → cpal Audio) wird als statische Library gebaut und via C-ABI/FFI in Swift eingebunden. SwiftUI rendert Spektrum und Wasserfall via Metal mit 60fps.

**Tech Stack:** Rust 1.78+, SoapySDR (Homebrew), RustFFT 6, cpal 0.15, cbindgen 0.27, Swift 5.10, SwiftUI, MetalKit, Xcode 15+

---

## Voraussetzungen (einmalig, manuell)

```bash
# Rust installieren
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# SoapySDR + Treiber via Homebrew
brew install soapysdr
brew install hackrf
brew install librtlsdr

# cbindgen global installieren
cargo install cbindgen

# Prüfen ob SoapySDR gefunden wird
pkg-config --libs SoapySDR
# Erwartet: -L/opt/homebrew/lib -lSoapySDR
```

Xcode 15+ muss installiert sein (aus dem Mac App Store).

---

## Dateistruktur

```
SDRapp/
├── sdrapp-core/                    # Rust Crate
│   ├── src/
│   │   ├── lib.rs                  # C-ABI Export (pub extern "C" Funktionen)
│   │   ├── device.rs               # SoapySDR-Wrapper
│   │   ├── fft.rs                  # FFT-Pipeline (RustFFT)
│   │   ├── demod.rs                # AM + WBFM Demodulator
│   │   ├── audio.rs                # cpal Audio-Ausgabe
│   │   └── pipeline.rs             # Verbindet alle Komponenten
│   ├── build.rs                    # cbindgen: generiert sdrapp_core.h
│   └── Cargo.toml
├── SDRapp/                         # Xcode SwiftUI App
│   ├── App/
│   │   ├── SDRappApp.swift         # @main Entry Point
│   │   └── AppState.swift          # @Observable globaler State
│   ├── Application/
│   │   ├── SDRCore.swift           # Swift-Wrapper um Rust-FFI
│   │   ├── DeviceManager.swift     # Geräte-Enumeration
│   │   └── sdrapp_core.h           # cbindgen-Output (auto-generiert)
│   ├── Views/
│   │   ├── ContentView.swift       # Root NavigationSplitView
│   │   ├── Sidebar/
│   │   │   ├── SidebarView.swift
│   │   │   ├── DevicePickerView.swift
│   │   │   ├── ModePickerView.swift
│   │   │   └── GainControlView.swift
│   │   └── Spectrum/
│   │       ├── SpectrumContainerView.swift
│   │       ├── SpectrumRenderer.swift   # MTKView Renderer für Spektrum
│   │       └── WaterfallRenderer.swift  # MTKView Renderer für Wasserfall
│   ├── Metal/
│   │   ├── Spectrum.metal
│   │   └── Waterfall.metal
│   └── Resources/
│       └── Assets.xcassets
└── docs/
```

---

## Task 1: Rust Core — Projekt-Setup

**Files:**
- Create: `sdrapp-core/Cargo.toml`
- Create: `sdrapp-core/build.rs`
- Create: `sdrapp-core/src/lib.rs` (Stub)

- [ ] **Schritt 1: Rust-Crate initialisieren**

```bash
cd /Users/jarodschilke/Documents/Projekte/SDRapp
cargo new --lib sdrapp-core
cd sdrapp-core
```

- [ ] **Schritt 2: `Cargo.toml` schreiben**

```toml
[package]
name = "sdrapp-core"
version = "0.1.0"
edition = "2021"

[lib]
name = "sdrapp_core"
crate-type = ["staticlib"]

[dependencies]
soapysdr = "0.3"
rustfft = "6"
ringbuf = "0.4"
cpal = { version = "0.15", features = ["default"] }
num-complex = "0.4"

[build-dependencies]
cbindgen = "0.27"

[dev-dependencies]
approx = "0.5"
```

- [ ] **Schritt 3: `build.rs` schreiben**

```rust
fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = format!("{}/../SDRapp/Application", crate_dir);
    std::fs::create_dir_all(&output_dir).ok();
    
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("SDRAPP_CORE_H")
        .generate()
        .expect("cbindgen fehlgeschlagen")
        .write_to_file(format!("{}/sdrapp_core.h", output_dir));
    
    // SoapySDR via pkg-config verlinken
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-lib=SoapySDR");
}
```

- [ ] **Schritt 4: Stub `src/lib.rs` anlegen**

```rust
mod device;
mod fft;
mod demod;
mod audio;
mod pipeline;

pub use pipeline::SdrappCore;
```

- [ ] **Schritt 5: Stub-Module anlegen damit es kompiliert**

```bash
touch src/device.rs src/fft.rs src/demod.rs src/audio.rs src/pipeline.rs
```

Inhalt jeder Datei zunächst nur: `// placeholder`

- [ ] **Schritt 6: Build prüfen**

```bash
cargo build 2>&1 | head -30
```

Erwartet: Kompiliert (ggf. Warnings wegen leerer Module, keine Errors).

- [ ] **Schritt 7: Commit**

```bash
cd /Users/jarodschilke/Documents/Projekte/SDRapp
git add sdrapp-core/
git commit -m "feat(core): initial Rust crate setup with dependencies"
```

---

## Task 2: Ring Buffer

**Files:**
- Modify: `sdrapp-core/src/fft.rs`

Der Ring Buffer verbindet den SoapySDR-Empfänger-Thread mit der FFT-Pipeline. Er braucht keine eigene Datei — `ringbuf` Crate wird direkt in `pipeline.rs` benutzt. Hier testen wir die FFT-Eingabe-Vorbereitung.

- [ ] **Schritt 1: Test schreiben (`src/fft.rs`)**

```rust
use num_complex::Complex;

pub const FFT_SIZE: usize = 1024;

/// Berechnet FFT-Magnitude in dBm aus IQ-Samples.
/// Eingabe: FFT_SIZE komplexe Samples
/// Ausgabe: FFT_SIZE Magnitude-Werte in dBm (typisch -120..0)
pub fn compute_fft_magnitude(samples: &[Complex<f32>], out: &mut [f32]) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_fft_sine_peak() {
        // Sinus bei Bin 10 → Peak bei Index 10
        let mut samples: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|i| {
                let phase = 2.0 * PI * 10.0 * i as f32 / FFT_SIZE as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();
        
        let mut magnitude = vec![0.0f32; FFT_SIZE];
        compute_fft_magnitude(&samples, &mut magnitude);
        
        // Peak muss bei Index 10 sein
        let peak_idx = magnitude
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(peak_idx, 10);
        
        // Peak muss deutlich über Rauschen liegen
        assert!(magnitude[10] > magnitude[5] + 20.0,
            "Peak bei Bin 10 erwartet, got peak={} noise={}", magnitude[10], magnitude[5]);
    }

    #[test]
    fn test_fft_output_range() {
        let samples: Vec<Complex<f32>> = vec![Complex::new(0.5, 0.0); FFT_SIZE];
        let mut magnitude = vec![0.0f32; FFT_SIZE];
        compute_fft_magnitude(&samples, &mut magnitude);
        // Alle Werte sollen im vernünftigen dBm-Bereich sein
        for &v in &magnitude {
            assert!(v > -200.0 && v < 10.0, "Unerwarteter dBm-Wert: {}", v);
        }
    }
}
```

- [ ] **Schritt 2: Test ausführen — muss fehlschlagen**

```bash
cd sdrapp-core && cargo test test_fft_sine_peak 2>&1 | tail -10
```

Erwartet: FAILED (unimplemented)

- [ ] **Schritt 3: Implementierung in `src/fft.rs`**

```rust
use num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::Arc;

pub const FFT_SIZE: usize = 1024;

pub struct FftProcessor {
    fft: Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl FftProcessor {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        Self { fft, scratch }
    }

    pub fn process(&mut self, samples: &[Complex<f32>], out: &mut [f32]) {
        assert_eq!(samples.len(), FFT_SIZE);
        assert_eq!(out.len(), FFT_SIZE);

        let mut buf: Vec<Complex<f32>> = samples.iter()
            .enumerate()
            .map(|(i, &s)| {
                // Hann-Fenster
                let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32
                    / (FFT_SIZE - 1) as f32).cos());
                s * w
            })
            .collect();

        self.fft.process_with_scratch(&mut buf, &mut self.scratch);

        // FFT-Shift: negative Frequenzen nach links
        let half = FFT_SIZE / 2;
        for i in 0..FFT_SIZE {
            let shifted = (i + half) % FFT_SIZE;
            let mag = buf[shifted].norm();
            // dBm: 10 * log10(mag^2 / FFT_SIZE) - normalisiert
            out[i] = if mag > 0.0 {
                20.0 * mag.log10() - 10.0 * (FFT_SIZE as f32).log10()
            } else {
                -120.0
            };
            out[i] = out[i].max(-120.0).min(0.0);
        }
    }
}

/// Convenience-Funktion für Tests und C-ABI
pub fn compute_fft_magnitude(samples: &[Complex<f32>], out: &mut [f32]) {
    let mut proc = FftProcessor::new();
    proc.process(samples, out);
}
```

- [ ] **Schritt 4: Tests ausführen — müssen bestehen**

```bash
cargo test fft 2>&1 | tail -15
```

Erwartet: `test tests::test_fft_sine_peak ... ok` und `test tests::test_fft_output_range ... ok`

- [ ] **Schritt 5: Commit**

```bash
git add sdrapp-core/src/fft.rs
git commit -m "feat(core/fft): FFT pipeline with Hann window and dBm normalization"
```

---

## Task 3: Demodulator (AM + WBFM)

**Files:**
- Modify: `sdrapp-core/src/demod.rs`

- [ ] **Schritt 1: Tests schreiben**

```rust
// src/demod.rs
use num_complex::Complex;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum DemodMode {
    Am = 0,
    Wbfm = 1,
}

pub struct Demodulator {
    mode: DemodMode,
    prev_sample: Complex<f32>,
    // FM-Zustand
    sample_rate: f32,
    audio_rate: f32,
    decimation: usize,
    decimation_counter: usize,
}

impl Demodulator {
    pub fn new(mode: DemodMode, sample_rate: f32) -> Self {
        let audio_rate = 44100.0;
        let decimation = (sample_rate / audio_rate).round() as usize;
        Self {
            mode,
            prev_sample: Complex::new(1.0, 0.0),
            sample_rate,
            audio_rate,
            decimation: decimation.max(1),
            decimation_counter: 0,
        }
    }

    /// Demoduliert IQ-Samples → Audio-Samples (f32, -1..1)
    pub fn process(&mut self, samples: &[Complex<f32>]) -> Vec<f32> {
        match self.mode {
            DemodMode::Am => self.demod_am(samples),
            DemodMode::Wbfm => self.demod_wbfm(samples),
        }
    }

    fn demod_am(&self, samples: &[Complex<f32>]) -> Vec<f32> {
        // AM: Hüllkurve (Betrag), dann Dezimieren
        samples.iter()
            .step_by(self.decimation)
            .map(|s| s.norm().min(1.0))
            .collect()
    }

    fn demod_wbfm(&mut self, samples: &[Complex<f32>]) -> Vec<f32> {
        // WBFM: Phasendifferenz zwischen aufeinanderfolgenden Samples
        let mut audio = Vec::with_capacity(samples.len() / self.decimation + 1);
        for (i, &s) in samples.iter().enumerate() {
            let demod = (s * self.prev_sample.conj()).arg() / std::f32::consts::PI;
            self.prev_sample = s;
            self.decimation_counter += 1;
            if self.decimation_counter >= self.decimation {
                self.decimation_counter = 0;
                audio.push(demod.clamp(-1.0, 1.0));
            }
        }
        audio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_am_envelope() {
        // AM: konstante Amplitude 0.5 → Audio ~0.5
        let samples: Vec<Complex<f32>> = (0..2048)
            .map(|_| Complex::new(0.5, 0.0))
            .collect();
        let mut demod = Demodulator::new(DemodMode::Am, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(!audio.is_empty());
        for &s in &audio {
            assert!((s - 0.5).abs() < 0.01, "AM Hüllkurve falsch: {}", s);
        }
    }

    #[test]
    fn test_wbfm_silence() {
        // WBFM: konstante Phase → kein Phasensprung → Stille (~0)
        let samples: Vec<Complex<f32>> = (0..2048)
            .map(|_| Complex::new(1.0, 0.0))
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        assert!(!audio.is_empty());
        let mean: f32 = audio.iter().sum::<f32>() / audio.len() as f32;
        assert!(mean.abs() < 0.05, "WBFM-Stille erwartet, got mean={}", mean);
    }

    #[test]
    fn test_audio_range() {
        let samples: Vec<Complex<f32>> = (0..4096)
            .map(|i| {
                let phase = 0.01 * i as f32;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();
        let mut demod = Demodulator::new(DemodMode::Wbfm, 2_048_000.0);
        let audio = demod.process(&samples);
        for &s in &audio {
            assert!(s >= -1.0 && s <= 1.0, "Audio außerhalb -1..1: {}", s);
        }
    }
}
```

- [ ] **Schritt 2: Test ausführen — muss fehlschlagen**

```bash
cargo test demod 2>&1 | tail -10
```

Erwartet: FAILED (leere Datei)

- [ ] **Schritt 3: Implementierung ist bereits oben enthalten** — die Tests und die Implementierung sind in derselben Datei. Schreibe jetzt die vollständige Datei wie oben.

- [ ] **Schritt 4: Tests ausführen**

```bash
cargo test demod 2>&1 | tail -15
```

Erwartet: alle 3 Tests grün.

- [ ] **Schritt 5: Commit**

```bash
git add sdrapp-core/src/demod.rs
git commit -m "feat(core/demod): AM envelope and WBFM discriminator with decimation"
```

---

## Task 4: SoapySDR Device Layer

**Files:**
- Modify: `sdrapp-core/src/device.rs`

- [ ] **Schritt 1: `src/device.rs` schreiben**

```rust
use soapysdr::Direction::Rx;
use num_complex::Complex;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub label: String,
    pub args: String,
    pub driver: String,
}

pub struct SdrDevice {
    device: soapysdr::Device,
    sample_rate: f64,
}

impl SdrDevice {
    /// Gibt alle angeschlossenen SoapySDR-Geräte zurück.
    /// Gibt leere Liste zurück wenn keine Hardware gefunden.
    pub fn enumerate() -> Vec<DeviceInfo> {
        match soapysdr::enumerate("") {
            Ok(list) => list.iter().map(|args| DeviceInfo {
                label: args.get("label").unwrap_or("Unknown").to_string(),
                args: args.to_string(),
                driver: args.get("driver").unwrap_or("unknown").to_string(),
            }).collect(),
            Err(_) => vec![],
        }
    }

    pub fn open(args: &str) -> Result<Self, soapysdr::Error> {
        let device = soapysdr::Device::new(args)?;
        Ok(Self { device, sample_rate: 2_048_000.0 })
    }

    pub fn configure(&self, frequency_hz: u64, gain_db: f64, sample_rate: f64)
        -> Result<(), soapysdr::Error>
    {
        self.device.set_sample_rate(Rx, 0, sample_rate)?;
        self.device.set_frequency(Rx, 0, frequency_hz as f64, ())?;
        self.device.set_gain(Rx, 0, gain_db)?;
        Ok(())
    }

    pub fn sample_rate(&self) -> f64 { self.sample_rate }

    /// Gibt einen RX-Stream zurück der IQ-Samples liefert.
    pub fn rx_stream(&self) -> Result<soapysdr::RxStream<Complex<f32>>, soapysdr::Error> {
        self.device.rx_stream(&[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_does_not_panic() {
        // Gibt entweder Geräte oder leere Liste — darf nicht paniken
        let devices = SdrDevice::enumerate();
        println!("Gefundene Geräte: {}", devices.len());
        // Kein assert — Hardware ggf. nicht angeschlossen
    }

    #[test]
    fn test_open_invalid_args_returns_error() {
        let result = SdrDevice::open("driver=nonexistent_xyz");
        assert!(result.is_err(), "Ungültige Args sollten Fehler zurückgeben");
    }
}
```

- [ ] **Schritt 2: Tests ausführen**

```bash
cargo test device 2>&1 | tail -15
```

Erwartet: `test_enumerate_does_not_panic ... ok`, `test_open_invalid_args_returns_error ... ok`

- [ ] **Schritt 3: Commit**

```bash
git add sdrapp-core/src/device.rs
git commit -m "feat(core/device): SoapySDR enumeration and device wrapper"
```

---

## Task 5: Audio-Ausgabe

**Files:**
- Modify: `sdrapp-core/src/audio.rs`

- [ ] **Schritt 1: `src/audio.rs` schreiben**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, HeapConsumer, HeapProducer};
use std::sync::{Arc, Mutex};

pub struct AudioOutput {
    _stream: cpal::Stream,  // Stream muss am Leben bleiben
    pub producer: HeapProducer<f32>,
}

impl AudioOutput {
    pub fn new(sample_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("Kein Audio-Ausgabegerät gefunden")?;
        
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let rb = HeapRb::<f32>::new(sample_rate as usize); // 1 Sekunde Buffer
        let (producer, mut consumer) = rb.split();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                for sample in data.iter_mut() {
                    *sample = consumer.pop().unwrap_or(0.0);
                }
            },
            |err| eprintln!("Audio-Fehler: {}", err),
            None,
        )?;

        stream.play()?;
        Ok(Self { _stream: stream, producer })
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for &s in samples {
            // Blockiert nicht — voller Buffer wird ignoriert (Drop)
            let _ = self.producer.push(s);
        }
    }
}
```

- [ ] **Schritt 2: Build prüfen (kein Unit-Test für Audio-Hardware)**

```bash
cargo build 2>&1 | grep -E "error|warning" | head -20
```

Erwartet: Kompiliert ohne Errors.

- [ ] **Schritt 3: Commit**

```bash
git add sdrapp-core/src/audio.rs
git commit -m "feat(core/audio): cpal audio output with ring buffer"
```

---

## Task 6: Signal-Pipeline

**Files:**
- Modify: `sdrapp-core/src/pipeline.rs`
- Modify: `sdrapp-core/src/lib.rs`

Die Pipeline verbindet: Device → Ring Buffer → FFT → Demodulator → Audio.

- [ ] **Schritt 1: `src/pipeline.rs` schreiben**

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use num_complex::Complex;
use ringbuf::{HeapRb, HeapProducer, HeapConsumer};

use crate::device::{DeviceInfo, SdrDevice};
use crate::fft::{FftProcessor, FFT_SIZE};
use crate::demod::{Demodulator, DemodMode};
use crate::audio::AudioOutput;

const SAMPLE_RATE: f64 = 2_048_000.0;
const AUDIO_RATE: u32 = 44_100;

/// Shared state zwischen Empfänger-Thread und Swift-UI-Thread
struct SharedState {
    fft_data: [f32; FFT_SIZE],
    is_running: bool,
}

pub struct SdrappCore {
    state: Arc<Mutex<SharedState>>,
    // IQ-Sample Ring Buffer Producer (Empfänger-Thread schreibt)
    iq_producer: Option<HeapProducer<Complex<f32>>>,
    device_args: Option<String>,
    frequency_hz: u64,
    gain_db: f64,
    demod_mode: DemodMode,
}

impl SdrappCore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SharedState {
                fft_data: [-120.0; FFT_SIZE],
                is_running: false,
            })),
            iq_producer: None,
            device_args: None,
            frequency_hz: 100_000_000, // 100 MHz
            gain_db: 30.0,
            demod_mode: DemodMode::Wbfm,
        }
    }

    pub fn list_devices() -> Vec<DeviceInfo> {
        SdrDevice::enumerate()
    }

    pub fn set_device(&mut self, args: &str) {
        self.device_args = Some(args.to_string());
    }

    pub fn set_frequency(&mut self, hz: u64) {
        self.frequency_hz = hz;
    }

    pub fn set_gain(&mut self, db: f64) {
        self.gain_db = db;
    }

    pub fn set_demod(&mut self, mode: DemodMode) {
        self.demod_mode = mode;
    }

    /// Kopiert aktuelle FFT-Daten in out_buf. Gibt FFT_SIZE zurück.
    pub fn get_fft(&self, out_buf: &mut [f32]) -> usize {
        let state = self.state.lock().unwrap();
        let len = out_buf.len().min(FFT_SIZE);
        out_buf[..len].copy_from_slice(&state.fft_data[..len]);
        len
    }

    /// Startet Empfänger- und DSP-Thread.
    pub fn start(&mut self) -> bool {
        let device_args = match &self.device_args {
            Some(a) => a.clone(),
            None => {
                // Kein Gerät ausgewählt
                return false;
            }
        };

        let device = match SdrDevice::open(&device_args) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Gerät öffnen fehlgeschlagen: {}", e);
                return false;
            }
        };

        if let Err(e) = device.configure(self.frequency_hz, self.gain_db, SAMPLE_RATE) {
            eprintln!("Gerät konfigurieren fehlgeschlagen: {}", e);
            return false;
        }

        let rb = HeapRb::<Complex<f32>>::new(SAMPLE_RATE as usize); // 1s Buffer
        let (iq_producer, mut iq_consumer) = rb.split();
        self.iq_producer = Some(iq_producer);

        let state = Arc::clone(&self.state);
        let demod_mode = self.demod_mode;

        // DSP-Thread: liest IQ aus Ring Buffer, rechnet FFT + Demod
        thread::spawn(move || {
            let mut fft = FftProcessor::new();
            let mut demod = Demodulator::new(demod_mode, SAMPLE_RATE as f32);
            let mut audio = match AudioOutput::new(AUDIO_RATE) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Audio-Init fehlgeschlagen: {}", e);
                    return;
                }
            };
            let mut buf = vec![Complex::default(); FFT_SIZE];
            let mut fft_out = vec![0.0f32; FFT_SIZE];
            let mut overflow = 0usize;

            loop {
                {
                    let s = state.lock().unwrap();
                    if !s.is_running { break; }
                }

                // Warte bis genug Samples im Buffer
                let available = iq_consumer.len();
                if available < FFT_SIZE {
                    thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }

                // FFT_SIZE Samples lesen
                for s in buf.iter_mut() {
                    *s = iq_consumer.pop().unwrap_or_default();
                }

                // FFT
                fft.process(&buf, &mut fft_out);
                {
                    let mut s = state.lock().unwrap();
                    s.fft_data.copy_from_slice(&fft_out);
                }

                // Demodulation + Audio
                let audio_samples = demod.process(&buf);
                audio.push_samples(&audio_samples);
            }
        });

        // Empfänger-Thread: liest von SoapySDR in Ring Buffer
        // (vereinfacht: producer wird in closure gemoved)
        // In echter Implementierung: producer wird via Arc<Mutex> geteilt
        // Für Phase 1: Empfänger-Thread in start() inline
        {
            let mut state_guard = self.state.lock().unwrap();
            state_guard.is_running = true;
        }

        // Empfang-Thread
        let state2 = Arc::clone(&self.state);
        // Note: producer hier aus self nehmen geht nicht direkt wegen Ownership.
        // Lösung: producer via Arc<Mutex<Option<Producer>>> teilen.
        // Für Phase 1 akzeptabel: Start gibt true zurück, echter Thread wird beim
        // vollständigen Xcode-Build ergänzt (siehe Task 12 Integration).

        true
    }

    pub fn stop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_running = false;
        self.iq_producer = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_core_default_state() {
        let core = SdrappCore::new();
        let mut buf = vec![0.0f32; FFT_SIZE];
        let len = core.get_fft(&mut buf);
        assert_eq!(len, FFT_SIZE);
        // Alle Werte sollten -120 (Rauschen) sein
        assert!(buf.iter().all(|&v| v <= -119.0));
    }

    #[test]
    fn test_list_devices_no_panic() {
        let devices = SdrappCore::list_devices();
        println!("Gefundene Geräte: {}", devices.len());
    }
}
```

- [ ] **Schritt 2: `src/lib.rs` aktualisieren**

```rust
mod device;
mod fft;
mod demod;
mod audio;
mod pipeline;

pub use pipeline::SdrappCore;
pub use fft::FFT_SIZE;
pub use demod::DemodMode;
```

- [ ] **Schritt 3: Tests laufen lassen**

```bash
cargo test pipeline 2>&1 | tail -15
```

Erwartet: `test_new_core_default_state ... ok`, `test_list_devices_no_panic ... ok`

- [ ] **Schritt 4: Commit**

```bash
git add sdrapp-core/src/pipeline.rs sdrapp-core/src/lib.rs
git commit -m "feat(core/pipeline): signal processing pipeline connecting device, FFT, demod, audio"
```

---

## Task 7: C-ABI Export

**Files:**
- Modify: `sdrapp-core/src/lib.rs` (C-ABI Funktionen anhängen)

- [ ] **Schritt 1: C-ABI Funktionen zu `lib.rs` hinzufügen**

Anhängen nach den bestehenden `pub use` Zeilen:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque-Pointer für Swift
#[repr(C)]
pub struct SdrappCoreOpaque {
    _private: [u8; 0],
}

// cbindgen:ignore
static mut CORE_INSTANCE: Option<Box<SdrappCore>> = None;

#[no_mangle]
pub extern "C" fn sdrapp_create() -> *mut SdrappCore {
    Box::into_raw(Box::new(SdrappCore::new()))
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_destroy(ptr: *mut SdrappCore) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_device(ptr: *mut SdrappCore, args: *const c_char) {
    if ptr.is_null() || args.is_null() { return; }
    let args_str = CStr::from_ptr(args).to_string_lossy();
    (*ptr).set_device(&args_str);
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_frequency(ptr: *mut SdrappCore, hz: u64) {
    if !ptr.is_null() { (*ptr).set_frequency(hz); }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_gain(ptr: *mut SdrappCore, db: f64) {
    if !ptr.is_null() { (*ptr).set_gain(db); }
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_set_demod(ptr: *mut SdrappCore, mode: u32) {
    if ptr.is_null() { return; }
    let m = if mode == 0 { DemodMode::Am } else { DemodMode::Wbfm };
    (*ptr).set_demod(m);
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_start(ptr: *mut SdrappCore) -> bool {
    if ptr.is_null() { return false; }
    (*ptr).start()
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_stop(ptr: *mut SdrappCore) {
    if !ptr.is_null() { (*ptr).stop(); }
}

/// Kopiert FFT-Daten in out_buf. Gibt Anzahl geschriebener Werte zurück.
/// out_len muss >= 1024 sein.
#[no_mangle]
pub unsafe extern "C" fn sdrapp_get_fft(
    ptr: *const SdrappCore,
    out_buf: *mut f32,
    out_len: usize,
) -> usize {
    if ptr.is_null() || out_buf.is_null() { return 0; }
    let buf = std::slice::from_raw_parts_mut(out_buf, out_len);
    (*ptr).get_fft(buf)
}

/// Gibt Anzahl angeschlossener Geräte zurück.
/// label_buf wird mit den Labels gefüllt (null-terminiert, max label_buf_size Bytes pro Gerät).
#[no_mangle]
pub unsafe extern "C" fn sdrapp_list_devices(
    out_count: *mut usize,
) -> *mut DeviceListC {
    let devices = SdrappCore::list_devices();
    let count = devices.len();
    if !out_count.is_null() { *out_count = count; }
    
    let mut items: Vec<DeviceItemC> = devices.into_iter().map(|d| DeviceItemC {
        label: CString::new(d.label).unwrap_or_default().into_raw(),
        args:  CString::new(d.args).unwrap_or_default().into_raw(),
    }).collect();
    items.shrink_to_fit();
    let items_ptr = items.as_mut_ptr();
    std::mem::forget(items); // Ownership geht an DeviceListC über
    
    let list = Box::new(DeviceListC { count, items: items_ptr });
    Box::into_raw(list)
}

#[repr(C)]
pub struct DeviceItemC {
    pub label: *mut c_char,
    pub args: *mut c_char,
}

#[repr(C)]
pub struct DeviceListC {
    pub count: usize,
    pub items: *mut DeviceItemC,  // raw pointer — C-ABI sicher
}

#[no_mangle]
pub unsafe extern "C" fn sdrapp_free_device_list(ptr: *mut DeviceListC) {
    if ptr.is_null() { return; }
    let list = Box::from_raw(ptr);
    if !list.items.is_null() {
        let items = Vec::from_raw_parts(list.items, list.count, list.count);
        for item in &items {
            if !item.label.is_null() { drop(CString::from_raw(item.label)); }
            if !item.args.is_null()  { drop(CString::from_raw(item.args));  }
        }
    }
}
```

- [ ] **Schritt 2: Header generieren**

```bash
cd sdrapp-core
cargo build --release 2>&1 | tail -20
```

Prüfe ob `../SDRapp/Application/sdrapp_core.h` generiert wurde:

```bash
ls -la ../SDRapp/Application/sdrapp_core.h 2>/dev/null || echo "Header nicht gefunden — Verzeichnis noch nicht erstellt"
```

Das SDRapp/-Verzeichnis existiert noch nicht (Xcode-Projekt folgt in Task 8). Erstelle es temporär:

```bash
mkdir -p ../SDRapp/Application
cargo build --release 2>&1 | tail -10
ls ../SDRapp/Application/sdrapp_core.h
```

Erwartet: Header-Datei existiert.

- [ ] **Schritt 3: Static Library prüfen**

```bash
ls -lh target/release/libsdrapp_core.a
```

Erwartet: Datei existiert, Größe ~1-5 MB.

- [ ] **Schritt 4: Commit**

```bash
git add sdrapp-core/src/lib.rs
git commit -m "feat(core/ffi): C-ABI export for Swift FFI integration"
```

---

## Task 8: Xcode-Projekt erstellen

**Files:**
- Create: `SDRapp/` (Xcode-Projekt via Xcode GUI)

- [ ] **Schritt 1: Xcode-Projekt anlegen (GUI)**

1. Xcode öffnen → File → New → Project
2. **macOS** → **App** → Next
3. Product Name: `SDRapp`
4. Team: ggf. leer lassen (Development)
5. Organization Identifier: `me.epost` (oder deine Domain)
6. Bundle Identifier: `me.epost.SDRapp`
7. Interface: **SwiftUI**
8. Language: **Swift**
9. **Speicherort:** `/Users/jarodschilke/Documents/Projekte/SDRapp/`
10. **Source Control: NICHT aktivieren** (wir haben bereits Git)
11. Create

- [ ] **Schritt 2: Rust Build-Script als Xcode Build Phase hinzufügen**

In Xcode: SDRapp Target → Build Phases → "+" → New Run Script Phase → ganz oben ziehen (vor Compile Sources):

```bash
# Rust Core bauen
export PATH="$HOME/.cargo/bin:$PATH"
cd "$SRCROOT/../sdrapp-core"
cargo build --release
cp target/release/libsdrapp_core.a "$SRCROOT/Application/"
```

- [ ] **Schritt 3: Static Library und Header zu Xcode hinzufügen**

In Xcode: SDRapp Target → Build Settings:

| Setting | Wert |
|---------|------|
| Library Search Paths | `$(SRCROOT)/Application` |
| Other Linker Flags | `-lsdrapp_core -lSoapySDR -L/opt/homebrew/lib` |
| Swift Compiler - Search Paths → Import Paths | (leer) |
| Objective-C Bridging Header | `SDRapp/Application/SDRapp-Bridging-Header.h` |

- [ ] **Schritt 4: Bridging Header anlegen**

Datei `SDRapp/Application/SDRapp-Bridging-Header.h`:

```c
#ifndef SDRapp_Bridging_Header_h
#define SDRapp_Bridging_Header_h

#include "sdrapp_core.h"

#endif
```

- [ ] **Schritt 5: Build prüfen**

Cmd+B in Xcode. Erwartet: Build Succeeded (ggf. Warnungen, keine Errors).

- [ ] **Schritt 6: Commit**

```bash
cd /Users/jarodschilke/Documents/Projekte/SDRapp
git add SDRapp/
git commit -m "feat(app): initial Xcode project with Rust build phase"
```

---

## Task 9: Swift FFI Bridge (SDRCore)

**Files:**
- Create: `SDRapp/Application/SDRCore.swift`

- [ ] **Schritt 1: `SDRCore.swift` schreiben**

```swift
import Foundation

enum DemodMode: UInt32 {
    case am = 0
    case wbfm = 1
}

struct SDRDeviceInfo: Identifiable {
    let id = UUID()
    let label: String
    let args: String
}

final class SDRCore {
    private let ptr: UnsafeMutableRawPointer

    init() {
        ptr = UnsafeMutableRawPointer(sdrapp_create())
    }

    deinit {
        sdrapp_destroy(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func listDevices() -> [SDRDeviceInfo] {
        var count: Int = 0
        guard let listPtr = sdrapp_list_devices(&count) else { return [] }
        defer { sdrapp_free_device_list(listPtr) }

        var result: [SDRDeviceInfo] = []
        let itemsPtr = listPtr.pointee.items
        for i in 0..<count {
            let item = itemsPtr![i]
            let label = item.label.map { String(cString: $0) } ?? "Unknown"
            let args  = item.args.map  { String(cString: $0) } ?? ""
            result.append(SDRDeviceInfo(label: label, args: args))
        }
        return result
    }

    func setDevice(_ args: String) {
        sdrapp_set_device(ptr.assumingMemoryBound(to: SdrappCore.self), args)
    }

    func setFrequency(_ hz: UInt64) {
        sdrapp_set_frequency(ptr.assumingMemoryBound(to: SdrappCore.self), hz)
    }

    func setGain(_ db: Double) {
        sdrapp_set_gain(ptr.assumingMemoryBound(to: SdrappCore.self), db)
    }

    func setDemod(_ mode: DemodMode) {
        sdrapp_set_demod(ptr.assumingMemoryBound(to: SdrappCore.self), mode.rawValue)
    }

    @discardableResult
    func start() -> Bool {
        sdrapp_start(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func stop() {
        sdrapp_stop(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func getFFT(size: Int = 1024) -> [Float] {
        var buf = [Float](repeating: -120, count: size)
        sdrapp_get_fft(ptr.assumingMemoryBound(to: SdrappCore.self), &buf, size)
        return buf
    }
}
```

- [ ] **Schritt 2: Build prüfen**

Cmd+B. Erwartet: Kompiliert ohne Errors.

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/Application/SDRCore.swift SDRapp/Application/SDRapp-Bridging-Header.h
git commit -m "feat(app/ffi): Swift wrapper around Rust C-ABI (SDRCore)"
```

---

## Task 10: AppState + DeviceManager

**Files:**
- Create: `SDRapp/App/AppState.swift`
- Create: `SDRapp/Application/DeviceManager.swift`

- [ ] **Schritt 1: `AppState.swift` schreiben**

```swift
import SwiftUI
import Observation

@Observable
final class AppState {
    // Gerät
    var availableDevices: [SDRDeviceInfo] = []
    var selectedDeviceArgs: String? = nil

    // Empfangsparameter
    var frequencyHz: UInt64 = 100_000_000   // 100 MHz
    var gainDb: Double = 30.0
    var demodMode: DemodMode = .wbfm
    var bandwidthHz: UInt64 = 200_000

    // Laufzeit-Status
    var isRunning: Bool = false
    var fftData: [Float] = Array(repeating: -120, count: 1024)

    private let core = SDRCore()
    private var fftTimer: Timer?

    func refreshDevices() {
        availableDevices = core.listDevices()
    }

    func startReceiving() {
        guard let args = selectedDeviceArgs else { return }
        core.setDevice(args)
        core.setFrequency(frequencyHz)
        core.setGain(gainDb)
        core.setDemod(demodMode)
        isRunning = core.start()
        if isRunning { startFFTPolling() }
    }

    func stopReceiving() {
        core.stop()
        isRunning = false
        stopFFTPolling()
    }

    func tuneFrequency(_ hz: UInt64) {
        frequencyHz = hz
        core.setFrequency(hz)
    }

    func changeGain(_ db: Double) {
        gainDb = db
        core.setGain(db)
    }

    func changeDemod(_ mode: DemodMode) {
        demodMode = mode
        core.setDemod(mode)
    }

    private func startFFTPolling() {
        fftTimer = Timer.scheduledTimer(withTimeInterval: 1.0/30.0, repeats: true) { [weak self] _ in
            guard let self else { return }
            self.fftData = self.core.getFFT()
        }
    }

    private func stopFFTPolling() {
        fftTimer?.invalidate()
        fftTimer = nil
    }
}
```

- [ ] **Schritt 2: `SDRappApp.swift` aktualisieren**

```swift
import SwiftUI

@main
struct SDRappApp: App {
    @State private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(appState)
                .frame(minWidth: 900, minHeight: 600)
        }
        .windowStyle(.titleBar)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
```

- [ ] **Schritt 3: Build prüfen**

Cmd+B. Erwartet: Kompiliert ohne Errors.

- [ ] **Schritt 4: Commit**

```bash
git add SDRapp/App/AppState.swift SDRapp/App/SDRappApp.swift
git commit -m "feat(app/state): AppState with FFT polling and device management"
```

---

## Task 11: Root Layout — NavigationSplitView + Sidebar

**Files:**
- Modify: `SDRapp/ContentView.swift`
- Create: `SDRapp/Views/Sidebar/SidebarView.swift`
- Create: `SDRapp/Views/Sidebar/DevicePickerView.swift`
- Create: `SDRapp/Views/Sidebar/ModePickerView.swift`
- Create: `SDRapp/Views/Sidebar/GainControlView.swift`

- [ ] **Schritt 1: `ContentView.swift` schreiben**

```swift
import SwiftUI

struct ContentView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        NavigationSplitView {
            SidebarView()
                .navigationSplitViewColumnWidth(min: 200, ideal: 240, max: 280)
        } detail: {
            SpectrumContainerView()
        }
        .onAppear {
            appState.refreshDevices()
        }
    }
}
```

- [ ] **Schritt 2: `SidebarView.swift` schreiben**

```swift
import SwiftUI

struct SidebarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        @Bindable var state = appState
        List {
            Section("Gerät") {
                DevicePickerView()
            }
            Section("Empfang") {
                ModePickerView()
                GainControlView()
            }
            Section("Steuerung") {
                Button(appState.isRunning ? "Stopp" : "Start") {
                    if appState.isRunning {
                        appState.stopReceiving()
                    } else {
                        appState.startReceiving()
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(appState.isRunning ? .red : .blue)
                .disabled(appState.selectedDeviceArgs == nil)
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("SDRapp")
    }
}
```

- [ ] **Schritt 3: `DevicePickerView.swift` schreiben**

```swift
import SwiftUI

struct DevicePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        if appState.availableDevices.isEmpty {
            Text("Kein Gerät gefunden")
                .foregroundStyle(.secondary)
                .font(.caption)
        } else {
            Picker("Gerät", selection: Binding(
                get: { appState.selectedDeviceArgs },
                set: { appState.selectedDeviceArgs = $0 }
            )) {
                Text("Auswählen…").tag(String?.none)
                ForEach(appState.availableDevices) { device in
                    Text(device.label).tag(Optional(device.args))
                }
            }
            .pickerStyle(.menu)
        }
        Button("Aktualisieren") {
            appState.refreshDevices()
        }
        .font(.caption)
    }
}
```

- [ ] **Schritt 4: `ModePickerView.swift` schreiben**

```swift
import SwiftUI

struct ModePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        Picker("Modus", selection: Binding(
            get: { appState.demodMode },
            set: { appState.changeDemod($0) }
        )) {
            Text("WBFM").tag(DemodMode.wbfm)
            Text("AM").tag(DemodMode.am)
        }
        .pickerStyle(.segmented)
    }
}
```

- [ ] **Schritt 5: `GainControlView.swift` schreiben**

```swift
import SwiftUI

struct GainControlView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Gain")
                Spacer()
                Text("\(Int(appState.gainDb)) dB")
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }
            Slider(
                value: Binding(
                    get: { appState.gainDb },
                    set: { appState.changeGain($0) }
                ),
                in: 0...60,
                step: 1
            )
        }
    }
}
```

- [ ] **Schritt 6: Build + Preview prüfen**

Cmd+B. Dann `ContentView` in Canvas öffnen: Cmd+Option+Return.
Erwartet: Sidebar mit Abschnitten "Gerät", "Empfang", "Steuerung" sichtbar.

- [ ] **Schritt 7: Commit**

```bash
git add SDRapp/
git commit -m "feat(app/ui): NavigationSplitView layout with sidebar controls"
```

---

## Task 12: Metal Spektrum-Renderer

**Files:**
- Create: `SDRapp/Metal/Spectrum.metal`
- Create: `SDRapp/Views/Spectrum/SpectrumRenderer.swift`
- Create: `SDRapp/Views/Spectrum/SpectrumContainerView.swift`

- [ ] **Schritt 1: `Spectrum.metal` schreiben**

```metal
#include <metal_stdlib>
using namespace metal;

struct SpectrumVertex {
    float4 position [[position]];
    float4 color;
};

// Zeichnet FFT-Daten als Liniengraph
vertex SpectrumVertex spectrum_vertex(
    uint vid [[vertex_id]],
    constant float* fftData [[buffer(0)]],
    constant uint& count    [[buffer(1)]]
) {
    float x = float(vid) / float(count - 1) * 2.0 - 1.0;  // -1..+1
    float normalized = (fftData[vid] + 120.0) / 120.0;      // 0..1 (-120..0 dBm)
    float y = normalized * 1.8 - 0.9;                        // -0.9..+0.9

    float4 color = float4(0.25, 0.65, 1.0, 1.0);
    return { float4(x, y, 0.0, 1.0), color };
}

fragment float4 spectrum_fragment(SpectrumVertex in [[stage_in]]) {
    return in.color;
}

// Füllt den Bereich unter dem Graphen (Triangle Strip)
vertex SpectrumVertex spectrum_fill_vertex(
    uint vid [[vertex_id]],
    constant float* fftData [[buffer(0)]],
    constant uint& count    [[buffer(1)]]
) {
    uint bin = vid / 2;
    float x = float(bin) / float(count - 1) * 2.0 - 1.0;
    float normalized = (fftData[bin] + 120.0) / 120.0;
    float yTop = normalized * 1.8 - 0.9;
    float y = (vid % 2 == 0) ? yTop : -0.9;
    float alpha = (vid % 2 == 0) ? 0.3 : 0.0;
    return { float4(x, y, 0.0, 1.0), float4(0.25, 0.65, 1.0, alpha) };
}
```

- [ ] **Schritt 2: `SpectrumRenderer.swift` schreiben**

```swift
import MetalKit
import simd

final class SpectrumRenderer: NSObject, MTKViewDelegate {
    private let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private var linePipeline: MTLRenderPipelineState
    private var fillPipeline: MTLRenderPipelineState
    private var fftBuffer: MTLBuffer
    private let fftSize: Int = 1024

    var fftData: [Float] = Array(repeating: -120, count: 1024)

    init?(mtkView: MTKView) {
        guard
            let device = MTLCreateSystemDefaultDevice(),
            let queue = device.makeCommandQueue()
        else { return nil }

        self.device = device
        self.commandQueue = queue
        mtkView.device = device
        mtkView.clearColor = MTLClearColor(red: 0.05, green: 0.05, blue: 0.08, alpha: 1)
        mtkView.colorPixelFormat = .bgra8Unorm

        guard let library = device.makeDefaultLibrary() else { return nil }

        let descriptor = MTLRenderPipelineDescriptor()
        descriptor.colorAttachments[0].pixelFormat = .bgra8Unorm
        descriptor.colorAttachments[0].isBlendingEnabled = true
        descriptor.colorAttachments[0].sourceRGBBlendFactor = .sourceAlpha
        descriptor.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha

        // Linie
        descriptor.vertexFunction   = library.makeFunction(name: "spectrum_vertex")
        descriptor.fragmentFunction = library.makeFunction(name: "spectrum_fragment")
        guard let line = try? device.makeRenderPipelineState(descriptor: descriptor) else { return nil }
        linePipeline = line

        // Füllung
        descriptor.vertexFunction = library.makeFunction(name: "spectrum_fill_vertex")
        guard let fill = try? device.makeRenderPipelineState(descriptor: descriptor) else { return nil }
        fillPipeline = fill

        guard let buf = device.makeBuffer(length: fftSize * MemoryLayout<Float>.stride,
                                          options: .storageModeShared) else { return nil }
        fftBuffer = buf

        super.init()
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        // FFT-Daten in Metal-Buffer kopieren
        let ptr = fftBuffer.contents().bindMemory(to: Float.self, capacity: fftSize)
        for i in 0..<fftSize { ptr[i] = fftData[i] }

        guard
            let drawable = view.currentDrawable,
            let passDescriptor = view.currentRenderPassDescriptor,
            let cmdBuffer = commandQueue.makeCommandBuffer(),
            let encoder = cmdBuffer.makeRenderCommandEncoder(descriptor: passDescriptor)
        else { return }

        var count = UInt32(fftSize)

        // Füllung zeichnen (Triangle Strip)
        encoder.setRenderPipelineState(fillPipeline)
        encoder.setVertexBuffer(fftBuffer, offset: 0, index: 0)
        encoder.setVertexBytes(&count, length: 4, index: 1)
        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: fftSize * 2)

        // Linie zeichnen (LineStrip)
        encoder.setRenderPipelineState(linePipeline)
        encoder.drawPrimitives(type: .lineStrip, vertexStart: 0, vertexCount: fftSize)

        encoder.endEncoding()
        cmdBuffer.present(drawable)
        cmdBuffer.commit()
    }
}
```

- [ ] **Schritt 3: `SpectrumContainerView.swift` schreiben**

```swift
import SwiftUI
import MetalKit

struct SpectrumContainerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(spacing: 0) {
            FrequencyBarView()
                .frame(height: 44)
            SpectrumMetalView(fftData: appState.fftData)
                .frame(maxWidth: .infinity, maxHeight: 200)
            WaterfallMetalView(fftData: appState.fftData)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Color(red: 0.05, green: 0.05, blue: 0.08))
    }
}

// MTKView-Wrapper für SwiftUI
struct SpectrumMetalView: NSViewRepresentable {
    var fftData: [Float]

    func makeNSView(context: Context) -> MTKView {
        let view = MTKView()
        view.preferredFramesPerSecond = 60
        view.isPaused = false
        view.enableSetNeedsDisplay = false
        context.coordinator.renderer = SpectrumRenderer(mtkView: view)
        view.delegate = context.coordinator.renderer
        return view
    }

    func updateNSView(_ view: MTKView, context: Context) {
        context.coordinator.renderer?.fftData = fftData
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    final class Coordinator {
        var renderer: SpectrumRenderer?
    }
}
```

- [ ] **Schritt 4: Stub für `FrequencyBarView` anlegen (kommt in Task 14)**

```swift
// SDRapp/Views/Spectrum/FrequencyBarView.swift
import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HStack {
            Text(formatFrequency(appState.frequencyHz))
                .font(.system(.title3, design: .monospaced))
                .foregroundStyle(.white)
            Spacer()
        }
        .padding(.horizontal, 12)
        .background(Color(red: 0.08, green: 0.08, blue: 0.12))
    }

    func formatFrequency(_ hz: UInt64) -> String {
        let mhz = Double(hz) / 1_000_000.0
        return String(format: "%.4f MHz", mhz)
    }
}
```

- [ ] **Schritt 5: Build prüfen**

Cmd+B. Erwartet: Kompiliert.

- [ ] **Schritt 6: Commit**

```bash
git add SDRapp/
git commit -m "feat(app/ui): Metal spectrum renderer with FFT visualization"
```

---

## Task 13: Metal Wasserfall-Renderer

**Files:**
- Create: `SDRapp/Metal/Waterfall.metal`
- Create: `SDRapp/Views/Spectrum/WaterfallRenderer.swift`
- Modify: `SDRapp/Views/Spectrum/SpectrumContainerView.swift` (WaterfallMetalView ergänzen)

- [ ] **Schritt 1: `Waterfall.metal` schreiben**

```metal
#include <metal_stdlib>
using namespace metal;

// Viridis-ähnliche Colormap (approximiert)
float4 viridis(float t) {
    t = clamp(t, 0.0, 1.0);
    float4 c0 = float4(0.267, 0.005, 0.329, 1.0);
    float4 c1 = float4(0.127, 0.566, 0.551, 1.0);
    float4 c2 = float4(0.993, 0.906, 0.144, 1.0);
    if (t < 0.5) return mix(c0, c1, t * 2.0);
    return mix(c1, c2, (t - 0.5) * 2.0);
}

// Compute: neue FFT-Zeile in Textur schreiben
kernel void waterfall_update(
    texture2d<float, access::write> tex [[texture(0)]],
    constant float* fftData            [[buffer(0)]],
    constant uint&  writeRow           [[buffer(1)]],
    constant uint&  fftCount           [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    if (tid >= fftCount) return;
    float normalized = (fftData[tid] + 120.0) / 120.0;
    tex.write(viridis(normalized), uint2(tid, writeRow));
}

// Vertex: Quad über den gesamten Bildschirm
struct WfVertex {
    float4 pos [[position]];
    float2 uv;
};

vertex WfVertex waterfall_vertex(uint vid [[vertex_id]]) {
    float2 positions[4] = { float2(-1,-1), float2(1,-1), float2(-1,1), float2(1,1) };
    float2 uvs[4]       = { float2(0,1),  float2(1,1),  float2(0,0),  float2(1,0) };
    return { float4(positions[vid], 0, 1), uvs[vid] };
}

// Fragment: sampelt Textur, versetzt um writeRow
fragment float4 waterfall_fragment(
    WfVertex in            [[stage_in]],
    texture2d<float> tex   [[texture(0)]],
    constant uint& writeRow [[buffer(0)]],
    constant uint& texHeight [[buffer(1)]]
) {
    constexpr sampler s(filter::nearest);
    // Offset damit aktuelle Zeile oben erscheint
    float rowOffset = float(writeRow) / float(texHeight);
    float2 uv = float2(in.uv.x, fract(in.uv.y + rowOffset));
    return tex.sample(s, uv);
}
```

- [ ] **Schritt 2: `WaterfallRenderer.swift` schreiben**

```swift
import MetalKit

final class WaterfallRenderer: NSObject, MTKViewDelegate {
    private let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private var computePipeline: MTLComputePipelineState
    private var renderPipeline: MTLRenderPipelineState
    private var texture: MTLTexture
    private var fftBuffer: MTLBuffer
    private let fftSize: Int = 1024
    private let textureHeight: Int = 512
    private var writeRow: UInt32 = 0

    var fftData: [Float] = Array(repeating: -120, count: 1024)

    init?(mtkView: MTKView) {
        guard
            let device = MTLCreateSystemDefaultDevice(),
            let queue = device.makeCommandQueue(),
            let library = device.makeDefaultLibrary()
        else { return nil }

        self.device = device
        self.commandQueue = queue
        mtkView.device = device
        mtkView.clearColor = MTLClearColor(red: 0.05, green: 0.05, blue: 0.08, alpha: 1)
        mtkView.colorPixelFormat = .bgra8Unorm

        // Compute Pipeline
        guard
            let computeFn = library.makeFunction(name: "waterfall_update"),
            let cp = try? device.makeComputePipelineState(function: computeFn)
        else { return nil }
        computePipeline = cp

        // Render Pipeline
        let rDesc = MTLRenderPipelineDescriptor()
        rDesc.colorAttachments[0].pixelFormat = .bgra8Unorm
        rDesc.vertexFunction   = library.makeFunction(name: "waterfall_vertex")
        rDesc.fragmentFunction = library.makeFunction(name: "waterfall_fragment")
        guard let rp = try? device.makeRenderPipelineState(descriptor: rDesc) else { return nil }
        renderPipeline = rp

        // Textur
        let texDesc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .rgba16Float,
            width: 1024, height: 512,
            mipmapped: false
        )
        texDesc.usage = [.shaderRead, .shaderWrite]
        texDesc.storageMode = .private
        guard let tex = device.makeTexture(descriptor: texDesc) else { return nil }
        texture = tex

        guard let buf = device.makeBuffer(length: 1024 * MemoryLayout<Float>.stride,
                                          options: .storageModeShared) else { return nil }
        fftBuffer = buf

        super.init()
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        // FFT-Daten in Buffer kopieren
        let ptr = fftBuffer.contents().bindMemory(to: Float.self, capacity: fftSize)
        for i in 0..<fftSize { ptr[i] = fftData[i] }

        guard
            let drawable = view.currentDrawable,
            let passDesc = view.currentRenderPassDescriptor,
            let cmdBuf = commandQueue.makeCommandBuffer()
        else { return }

        // 1. Compute: neue Zeile in Textur schreiben
        var row = writeRow
        var count = UInt32(fftSize)
        if let enc = cmdBuf.makeComputeCommandEncoder() {
            enc.setComputePipelineState(computePipeline)
            enc.setTexture(texture, index: 0)
            enc.setBuffer(fftBuffer, offset: 0, index: 0)
            enc.setBytes(&row, length: 4, index: 1)
            enc.setBytes(&count, length: 4, index: 2)
            let threads = MTLSize(width: fftSize, height: 1, depth: 1)
            let groups  = MTLSize(width: 1, height: 1, depth: 1)
            enc.dispatchThreads(threads, threadsPerThreadgroup: groups)
            enc.endEncoding()
        }

        writeRow = (writeRow + 1) % UInt32(textureHeight)

        // 2. Render: Textur als Quad zeichnen
        if let enc = cmdBuf.makeRenderCommandEncoder(descriptor: passDesc) {
            enc.setRenderPipelineState(renderPipeline)
            enc.setFragmentTexture(texture, index: 0)
            var h = UInt32(textureHeight)
            enc.setFragmentBytes(&row, length: 4, index: 0)
            enc.setFragmentBytes(&h,   length: 4, index: 1)
            enc.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
            enc.endEncoding()
        }

        cmdBuf.present(drawable)
        cmdBuf.commit()
    }
}
```

- [ ] **Schritt 3: `WaterfallMetalView` zu `SpectrumContainerView.swift` hinzufügen**

```swift
struct WaterfallMetalView: NSViewRepresentable {
    var fftData: [Float]

    func makeNSView(context: Context) -> MTKView {
        let view = MTKView()
        view.preferredFramesPerSecond = 30
        view.isPaused = false
        view.enableSetNeedsDisplay = false
        context.coordinator.renderer = WaterfallRenderer(mtkView: view)
        view.delegate = context.coordinator.renderer
        return view
    }

    func updateNSView(_ view: MTKView, context: Context) {
        context.coordinator.renderer?.fftData = fftData
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    final class Coordinator {
        var renderer: WaterfallRenderer?
    }
}
```

- [ ] **Schritt 4: Build prüfen**

Cmd+B. Erwartet: Kompiliert.

- [ ] **Schritt 5: Commit**

```bash
git add SDRapp/
git commit -m "feat(app/ui): Metal waterfall renderer with Viridis colormap"
```

---

## Task 14: Frequenz-Steuerung

**Files:**
- Modify: `SDRapp/Views/Spectrum/FrequencyBarView.swift`

- [ ] **Schritt 1: `FrequencyBarView.swift` vollständig schreiben**

```swift
import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState
    @State private var inputText: String = ""
    @State private var isEditing: Bool = false

    var body: some View {
        HStack(spacing: 12) {
            // Frequenz-Anzeige / Eingabe
            if isEditing {
                TextField("MHz", text: $inputText)
                    .font(.system(.title3, design: .monospaced))
                    .foregroundStyle(.white)
                    .textFieldStyle(.plain)
                    .onSubmit { commitFrequency() }
                    .onExitCommand { isEditing = false }
            } else {
                Text(formatFrequency(appState.frequencyHz))
                    .font(.system(.title3, design: .monospaced))
                    .foregroundStyle(.white)
                    .onTapGesture {
                        inputText = String(format: "%.4f", Double(appState.frequencyHz) / 1e6)
                        isEditing = true
                    }
            }

            Spacer()

            // Modus-Anzeige
            Text(appState.demodMode == .wbfm ? "WBFM" : "AM")
                .font(.caption)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color.blue.opacity(0.3))
                .cornerRadius(4)
                .foregroundStyle(.blue)

            // Status
            Circle()
                .fill(appState.isRunning ? Color.green : Color.gray)
                .frame(width: 8, height: 8)
        }
        .padding(.horizontal, 12)
        .frame(height: 44)
        .background(Color(red: 0.08, green: 0.08, blue: 0.12))
    }

    private func commitFrequency() {
        if let mhz = Double(inputText.replacingOccurrences(of: ",", with: ".")) {
            let hz = UInt64(mhz * 1_000_000)
            if hz >= 1_000 && hz <= 6_000_000_000 {  // 1 kHz bis 6 GHz
                appState.tuneFrequency(hz)
            }
        }
        isEditing = false
    }

    private func formatFrequency(_ hz: UInt64) -> String {
        String(format: "%.4f MHz", Double(hz) / 1_000_000.0)
    }
}
```

- [ ] **Schritt 2: Build + visuelle Prüfung**

Cmd+B, dann App starten (Cmd+R). Die Frequenzanzeige soll oben im Hauptbereich erscheinen und beim Klick editierbar sein.

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/Views/Spectrum/FrequencyBarView.swift
git commit -m "feat(app/ui): frequency display with tap-to-edit input"
```

---

## Task 15: Vollständige Empfänger-Thread Integration

**Files:**
- Modify: `sdrapp-core/src/pipeline.rs`

Der Empfänger-Thread in `start()` ist in Task 6 vereinfacht worden. Hier wird er vollständig implementiert.

- [ ] **Schritt 1: `pipeline.rs` — `start()` vollständig implementieren**

Ersetze die `start()`-Methode:

```rust
pub fn start(&mut self) -> bool {
    let device_args = match &self.device_args {
        Some(a) => a.clone(),
        None => return false,
    };

    let device = match SdrDevice::open(&device_args) {
        Ok(d) => d,
        Err(e) => { eprintln!("Gerät öffnen: {}", e); return false; }
    };

    if let Err(e) = device.configure(self.frequency_hz, self.gain_db, SAMPLE_RATE) {
        eprintln!("Gerät konfigurieren: {}", e); return false;
    }

    let mut rx_stream = match device.rx_stream() {
        Ok(s) => s,
        Err(e) => { eprintln!("RX-Stream: {}", e); return false; }
    };

    if let Err(e) = rx_stream.activate(None) {
        eprintln!("Stream aktivieren: {}", e); return false;
    }

    // IQ Ring Buffer
    let rb = HeapRb::<Complex<f32>>::new(SAMPLE_RATE as usize);
    let (mut iq_producer, mut iq_consumer) = rb.split();

    let state_rx = Arc::clone(&self.state);
    let state_dsp = Arc::clone(&self.state);
    let demod_mode = self.demod_mode;

    // Empfänger-Thread: SoapySDR → Ring Buffer
    thread::spawn(move || {
        let mut buf = vec![Complex::default(); 8192];
        loop {
            {
                if !state_rx.lock().unwrap().is_running { break; }
            }
            match rx_stream.read(&mut [&mut buf], 100_000) {
                Ok(len) => {
                    for &s in &buf[..len] {
                        let _ = iq_producer.push(s); // Drop on overflow
                    }
                }
                Err(e) => eprintln!("Read-Fehler: {}", e),
            }
        }
        let _ = rx_stream.deactivate(None);
    });

    // DSP-Thread: Ring Buffer → FFT → Demod → Audio
    thread::spawn(move || {
        let mut fft = FftProcessor::new();
        let mut demod = Demodulator::new(demod_mode, SAMPLE_RATE as f32);
        let mut audio = match AudioOutput::new(AUDIO_RATE) {
            Ok(a) => a,
            Err(e) => { eprintln!("Audio: {}", e); return; }
        };
        let mut buf = vec![Complex::default(); FFT_SIZE];
        let mut fft_out = vec![0.0f32; FFT_SIZE];

        loop {
            {
                if !state_dsp.lock().unwrap().is_running { break; }
            }
            if iq_consumer.len() < FFT_SIZE {
                thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            for s in buf.iter_mut() { *s = iq_consumer.pop().unwrap_or_default(); }
            fft.process(&buf, &mut fft_out);
            {
                let mut s = state_dsp.lock().unwrap();
                s.fft_data.copy_from_slice(&fft_out);
            }
            let audio_samples = demod.process(&buf);
            audio.push_samples(&audio_samples);
        }
    });

    self.state.lock().unwrap().is_running = true;
    true
}
```

- [ ] **Schritt 2: Cargo build**

```bash
cd sdrapp-core && cargo build --release 2>&1 | tail -20
```

Erwartet: Kompiliert ohne Errors.

- [ ] **Schritt 3: Alle Tests laufen**

```bash
cargo test 2>&1 | tail -20
```

Erwartet: Alle Tests grün (Hardware-Tests werden übersprungen wenn kein Gerät angeschlossen).

- [ ] **Schritt 4: Commit**

```bash
cd /Users/jarodschilke/Documents/Projekte/SDRapp
git add sdrapp-core/src/pipeline.rs
git commit -m "feat(core/pipeline): complete receiver and DSP thread implementation"
```

---

## Task 16: Erster vollständiger Test mit Hardware

- [ ] **Schritt 1: HackRF anschließen und prüfen**

```bash
hackrf_info
```

Erwartet: HackRF-Geräteinformationen werden angezeigt.

```bash
SoapySDRUtil --probe
```

Erwartet: HackRF erscheint in der Liste.

- [ ] **Schritt 2: App bauen und starten**

In Xcode: Cmd+R. App öffnet sich.

- [ ] **Schritt 3: Manueller Smoke-Test**

1. Sidebar: "Aktualisieren" klicken → HackRF erscheint in Geräteliste
2. HackRF auswählen
3. Modus: WBFM
4. Gain: 30 dB
5. Frequenz: `100.000 MHz` (lokaler UKW-Sender)
6. "Start" klicken
7. Erwartung:
   - Spektrum zeigt Signale
   - Wasserfall scrollt
   - Audio hörbar (UKW-Radio)
   - Status-LED grün

- [ ] **Schritt 4: RTL-SDR testen**

1. HackRF abziehen, RTL-SDR anschließen
2. "Aktualisieren" → RTL-SDR erscheint
3. Selbst Testen mit bekanntem Sender

- [ ] **Schritt 5: Abschluss-Commit**

```bash
git add .
git commit -m "feat: Phase 1 Foundation complete - SDRapp receiving FM radio"
```

- [ ] **Schritt 6: GitHub Push**

```bash
git push -u origin main
```

---

## Schnell-Referenz: Häufige Fehler

| Fehler | Ursache | Lösung |
|--------|---------|--------|
| `pkg-config: SoapySDR not found` | SoapySDR nicht installiert | `brew install soapysdr` |
| `hackrf_open() failed` | Treiber fehlt oder Gerät nicht berechtigt | `brew install hackrf`, USB-Berechtigung prüfen |
| `Audio output device not found` | macOS Audio-Berechtigung | System-Einstellungen → Datenschutz → Mikrofon |
| `linker: library not found: sdrapp_core` | Static Library nicht gebaut | Rust Release-Build in Xcode Build Phase prüfen |
| `cannot find type 'SdrappCore'` | Bridging-Header nicht gesetzt | Xcode Build Settings → Bridging Header Pfad prüfen |
| Wasserfall zeigt nur Schwarz | Metal-Textur nicht initialisiert | WaterfallRenderer init-Fehler in Console prüfen |

---

## Nächste Schritte nach Phase 1

Nach erfolgreichem Smoke-Test ist Phase 1 abgeschlossen. Die nächste Session startet mit dem Design-Doc für **Phase 2** (MVP: SSB/CW/NBFM, IQ-Aufnahme, Plugin-System, Presets, Onboarding).
