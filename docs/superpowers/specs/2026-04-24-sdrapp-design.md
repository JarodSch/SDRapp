# SDRapp — Design-Dokument

**Datum:** 2026-04-24  
**Status:** Genehmigt  
**Plattform:** macOS (Apple Silicon + Intel)  
**Lizenz:** Open Source  

---

## 1. Vision & Ziel

SDRapp ist eine moderne, native macOS-Anwendung für Software Defined Radio (SDR). Sie richtet sich an Einsteiger und erfahrene Funkamateure, die eine leistungsfähige SDR-Umgebung wollen, die sich wie eine echte Mac-App anfühlt — nicht wie ein Windows-Port.

**Das Kernproblem bestehender Tools** (SDR++, SDRangel, SDR-Console):
- Veraltete UI ohne native macOS-Integration
- Schlechte UX: zu viele versteckte Einstellungen, kein Onboarding, hohe Einstiegshürde

**SDRapp löst das durch:**
- Vollständig native macOS-UI mit SwiftUI
- Intuitive Sidebar-Navigation (macOS-Konvention: Finder, Mail, Xcode)
- Durchdachtes Onboarding für Einsteiger
- Maximale Performance durch Rust-DSP-Core mit Apple Silicon Support

---

## 2. Zielgruppe

**Primär:**
- **Einsteiger** — Menschen, die SDR entdecken wollen; brauchen Guidance, klare UI, gutes Onboarding
- **Funkamateure** — erfahrene Nutzer die Leistung und Flexibilität erwarten, aber bessere UX verdienen

**Nicht primär:** Profis/Forscher mit Spezialanforderungen (können aber durch Plugins bedient werden)

---

## 3. Hardware-Unterstützung

SDRapp nutzt **SoapySDR** als Abstraktionsschicht für maximale Gerätekompatibilität.

| Gerät | Priorität | Modus |
|-------|-----------|-------|
| HackRF One | Primär (Entwicklungshardware) | RX + TX (TX ab Phase 4) |
| RTL-SDR | Primär | RX |
| Airspy / Airspy Mini | Hoch | RX |
| SDRplay RSP | Hoch | RX |
| LimeSDR | Mittel | RX + TX |
| Alle SoapySDR-Geräte | Best-effort | je nach Gerät |

---

## 4. Architektur

SDRapp folgt einer **strikten 3-Schichten-Architektur** (Ansatz B). Jede Schicht hat eine klar definierte Verantwortlichkeit und kommuniziert nur mit der direkten Nachbarschicht.

```
┌─────────────────────────────────────────────┐
│           SwiftUI View Layer                │  Layer 3
│  Spektrum · Wasserfall · UI-Panels · Menus  │
└─────────────────┬───────────────────────────┘
                  │  Swift Observation / Combine
┌─────────────────▼───────────────────────────┐
│         Swift Application Layer             │  Layer 2
│  State · Plugins · Geräte-Manager · Bridge  │
└─────────────────┬───────────────────────────┘
                  │  C-ABI / FFI (cbindgen)
┌─────────────────▼───────────────────────────┐
│           Rust DSP Core                     │  Layer 1
│  SoapySDR · FFT · Demodulation · Filterung  │
└─────────────────┬───────────────────────────┘
                  │  SoapySDR C-API
┌─────────────────▼───────────────────────────┐
│               Hardware                      │
│  HackRF · RTL-SDR · Airspy · SDRplay · …   │
└─────────────────────────────────────────────┘
```

### 4.1 Rust DSP Core (Layer 1)

**Verantwortung:** Alle rechenintensiven Operationen. Kein UI-Wissen.

**Komponenten:**
- `soapysdr-rs` Binding — Gerätesteuerung, Sample-Akquisition
- Ring-Buffer — lock-freier IQ-Sample-Buffer zwischen Gerät und Verarbeitungs-Pipeline
- FFT-Pipeline — RustFFT, konfigurierbare FFT-Größe, Fenster-Funktionen (Hann, Blackman-Harris)
- Demodulator-Engine — AM, WBFM, NBFM, USB, LSB, CW (erweiterbar)
- Filter-Engine — FIR-Filter, einstellbare Bandbreite, Dezimation
- IQ-Recorder — Schreiben/Lesen von CF32/WAV-Dateien
- C-ABI Export — via `cbindgen` generierte Header, stabile öffentliche API

**Schnittstelle zu Layer 2:**
```c
// Beispiel C-ABI (wird von cbindgen generiert)
sdrapp_core_t* sdrapp_core_create(void);
void sdrapp_core_destroy(sdrapp_core_t*);
int  sdrapp_core_set_frequency(sdrapp_core_t*, uint64_t hz);
int  sdrapp_core_set_demod(sdrapp_core_t*, SdrappDemod mode);
int  sdrapp_core_get_fft(sdrapp_core_t*, float* out_buf, size_t len);
void sdrapp_core_start(sdrapp_core_t*);
void sdrapp_core_stop(sdrapp_core_t*);
```

### 4.2 Swift Application Layer (Layer 2)

**Verantwortung:** App-Logik, State-Management, Plugin-Orchestrierung. Vermittler zwischen Rust und UI.

**Komponenten:**
- `SDRCore` — Swift-Wrapper um die Rust-FFI-Bridge, Thread-safe
- `DeviceManager` — Geräte-Enumeration, Verbindung, Hot-Plug via SoapySDR
- `PluginManager` — Laden, Initialisieren und Orchestrieren von Swift-Package-Plugins
- `PresetManager` — Speichern/Laden von Frequenz-Presets und Konfigurationen
- `RecordingManager` — Steuerung von IQ-Aufnahmen, Dateiverwaltung
- `AppState` — globales Observable-Objekt (SwiftUI `@Observable`)

### 4.3 SwiftUI View Layer (Layer 3)

**Verantwortung:** Darstellung. Kein Business-Logik, kein direkter Rust-Zugriff.

**Layout:** `NavigationSplitView` — Sidebar links, Hauptinhalt rechts.

**Sidebar-Inhalt:**
- Geräte-Auswahl und Status
- Modus-Auswahl (AM/FM/SSB/CW/…)
- Gain, Bandbreite, Frequenz
- Plugin-Panels
- Preset-Liste

**Hauptbereich:**
- Spektrum-Display — Metal-Shader, 60fps, logarithmische dBm-Skala
- Wasserfall-Display — Metal-Shader, Echtzeit-Farbpaletten (Viridis, Inferno, klassisch)
- Frequenz-Leiste mit Klick-zum-Tunen

**Besonderheiten:**
- Dark Mode / Light Mode nativ unterstützt
- macOS Accessibility (VoiceOver-kompatible Controls)
- Keyboard-Shortcuts für alle häufigen Aktionen

---

## 5. Plugin-System

Plugins sind **Swift Packages** die das `SDRPlugin`-Protokoll implementieren.

```swift
public protocol SDRPlugin {
    var id: String { get }
    var displayName: String { get }
    func initialize(context: SDRPluginContext)
    func sidebarView() -> AnyView?      // optionaler Sidebar-Panel
    func onFFTData(_ data: [Float])     // read-only FFT-Daten
    func onAudioData(_ data: [Float])   // read-only Audio-Stream
}
```

**Plugins bekommen Zugriff auf:**
- FFT-Daten (read-only)
- Audio-Stream (read-only)
- Eigene Sidebar-Panel-Slots
- Frequenz-API (lesen + setzen)

**Plugins haben keinen Zugriff auf:**
- Rust-Core direkt
- Andere Plugin-Daten
- System-Ressourcen außerhalb ihrer Sandbox

DSP-Plugins können optional eine Rust-Library mitbringen, die über die Standard-FFI-Bridge eingehängt wird.

---

## 6. MVP-Featureumfang

### Muss enthalten sein (MVP):
- [ ] Geräte-Erkennung und -Verbindung (HackRF, RTL-SDR)
- [ ] Echtzeit-Spektrum-Anzeige (Metal, 60fps)
- [ ] Echtzeit-Wasserfall-Anzeige (Metal, 60fps)
- [ ] Frequenz-Tuning (Eingabe + Klick im Spektrum)
- [ ] Demodulation: AM, WBFM, NBFM, USB, LSB, CW
- [ ] Audio-Ausgabe (macOS CoreAudio)
- [ ] IQ-Aufnahme (CF32/WAV)
- [ ] IQ-Wiedergabe
- [ ] Plugin-System (Swift Package Protokoll)
- [ ] Preset-Verwaltung (Frequenzen + Einstellungen speichern)
- [ ] Frequenz-Bookmarks
- [ ] Onboarding-Flow für Erstnutzer
- [ ] Einstellungen-Fenster (macOS-Standard)

### Bewusst nicht im MVP:
- TX-Modus (HackRF)
- Digitale Modi (DMR, D-STAR, FT8)
- Remote-SDR / Netzwerkbetrieb
- Mehrkanal-Empfang

---

## 7. Technologie-Stack

| Bereich | Technologie | Begründung |
|---------|------------|------------|
| DSP / Signal | Rust | Performance, Speichersicherheit, Apple Silicon nativ |
| FFT | RustFFT | Beste Rust-FFT-Bibliothek, SIMD-optimiert |
| Hardware-Abstrak. | SoapySDR | Industriestandard, alle SDR-Geräte |
| FFI-Bridge | cbindgen | Automatische C-Header-Generierung aus Rust |
| UI-Framework | SwiftUI | Native macOS, Apple Silicon, NavigationSplitView |
| GPU-Rendering | Metal | Spektrum + Wasserfall mit 60fps |
| State Management | Swift @Observable | Native iOS/macOS Lösung, kein Drittframework |
| Build (Rust) | Cargo | Standard Rust Build-System |
| Build (Swift) | Xcode + SPM | Nativer macOS-Workflow |
| Versionskontrolle | Git / GitHub | SDRapp Repository |

---

## 8. Repository-Struktur

```
SDRapp/
├── SDRapp/                  # Xcode-Projekt (SwiftUI App)
│   ├── App/                 # App-Entry-Point, AppDelegate
│   ├── Views/               # SwiftUI Views
│   │   ├── Spectrum/        # Spektrum + Wasserfall (Metal)
│   │   ├── Sidebar/         # Sidebar-Panels
│   │   └── Onboarding/      # Onboarding-Flow
│   ├── Application/         # Layer 2: State, Manager
│   │   ├── SDRCore.swift
│   │   ├── DeviceManager.swift
│   │   ├── PluginManager.swift
│   │   └── PresetManager.swift
│   └── Resources/           # Assets, Lokalisierung
├── sdrapp-core/             # Rust Crate (Layer 1)
│   ├── src/
│   │   ├── lib.rs           # C-ABI Export
│   │   ├── device/          # SoapySDR-Binding
│   │   ├── dsp/             # FFT, Filter
│   │   ├── demod/           # Demodulator
│   │   └── recording/       # IQ-Aufnahme
│   ├── Cargo.toml
│   └── cbindgen.toml
├── SDRappPluginKit/         # Swift Package: Plugin-Protokoll
│   └── Sources/
│       └── SDRappPluginKit/
│           └── SDRPlugin.swift
├── docs/
│   ├── superpowers/specs/   # Design-Dokumente
│   └── plugin-development/  # Plugin-Entwickler-Guide
├── .gitignore
└── README.md
```

---

## 9. Entwicklungs-Roadmap

### Phase 1 — Foundation (~2–3 Monate)
**Ziel:** Radio hören. Spektrum sehen. App fühlt sich nativ an.

- Rust-Core: SoapySDR-Binding, FFT-Pipeline, Ring-Buffer, AM/WBFM-Demodulator, C-ABI
- SwiftUI: Sidebar-Layout, Metal-Spektrum-Renderer, Metal-Wasserfall-Renderer, Frequenz-Eingabe, Geräte-Auswahl
- FFI-Bridge zwischen Rust und Swift

### Phase 2 — Vollständiges MVP (~2–3 Monate)
**Ziel:** Öffentliche Alpha, Community-Feedback, Plugin-Entwickler einladen.

- DSP: SSB (USB/LSB), CW, NBFM, einstellbare Filter, IQ-Aufnahme/-Wiedergabe
- App: Plugin-System, Preset-Verwaltung, Frequenz-Bookmarks, Onboarding-Flow, Einstellungen-Fenster
- GitHub-Release: Alpha-Tag, README, Contributing-Guide

### Phase 3 — Reife (~3–6 Monate)
**Ziel:** Stabile Beta, aktive Community, SDR++ als macOS-Alternative ablösen.

- Geräte: Airspy, SDRplay, weitere SoapySDR-Geräte
- Features: Frequenz-Scanner, RDS-Dekoder (UKW-Radiotext), Mehrkanal-Empfang, Remote-SDR
- Qualität: Plugin-Marketplace, vollständige Dokumentation, Test-Suite, Crash-Reporting

### Phase 4+ — Zukunft (langfristig, feature-offen)
Ideen die nach Community-Feedback priorisiert werden:
- Digitale Modi: DMR, D-STAR, FT8, APRS, AIS
- ADS-B Flugzeug-Tracking
- Satelliten-Tracking / Doppler-Korrektur
- HackRF TX-Modus
- iCloud-Sync (Presets)
- iOS / iPadOS Companion App
- KI-gestützte Signalklassifikation (Core ML)
- Spektrum-Monitoring mit Alerts

---

## 10. Nicht-funktionale Anforderungen

| Anforderung | Ziel |
|------------|------|
| CPU-Last (Idle) | < 5% auf Apple M1 |
| FFT-Latenz | < 16ms (60fps) |
| Wasserfall-Framerate | 60fps |
| Speicherverbrauch | < 200MB RAM |
| App-Startzeit | < 2 Sekunden |
| macOS-Mindestversion | macOS 14 (Sonoma) |
| Architektur | Universal Binary (Apple Silicon + Intel) |

---

## 11. Offene Entscheidungen (für spätere Phasen)

- **App-Signing & Distribution:** GitHub Releases vs. Mac App Store
- **Plugin-Signierung:** Vertrauensmodell für Community-Plugins
- **Internationalisierung:** Welche Sprachen ab wann
- **Lizenz:** MIT vs. GPL (GNU) — Implikationen für Plugin-Ökosystem klären
