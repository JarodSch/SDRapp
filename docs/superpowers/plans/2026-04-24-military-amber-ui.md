# Military Amber UI Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the SDRapp macOS UI from default macOS appearance to a cohesive "Military Amber" dark theme with monospace typography.

**Architecture:** All changes are SwiftUI-only (no Rust/Metal logic changes). A central `Theme.swift` holds all color constants. `NavigationSplitView` is replaced with a plain `HSplitView`/`HStack` for full control. Metal spectrum line color changes from blue to amber in `Spectrum.metal`.

**Tech Stack:** SwiftUI, Metal (color constant only), SF Mono font, `xcodebuild` CLI for build verification.

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `SDRapp/SDRapp/Theme.swift` | **Create** | Color + font constants |
| `SDRapp/SDRapp/SDRappApp.swift` | **Modify** | Force `.preferredColorScheme(.dark)` |
| `SDRapp/SDRapp/ContentView.swift` | **Modify** | Replace `NavigationSplitView` with `HSplitView` |
| `SDRapp/SDRapp/Views/Sidebar/SidebarView.swift` | **Modify** | Custom dark amber sidebar layout |
| `SDRapp/SDRapp/Views/Sidebar/DevicePickerView.swift` | **Modify** | Amber-styled device picker + refresh button |
| `SDRapp/SDRapp/Views/Sidebar/ModePickerView.swift` | **Modify** | Custom two-button amber toggle |
| `SDRapp/SDRapp/Views/Sidebar/GainControlView.swift` | **Modify** | Custom amber slider |
| `SDRapp/SDRapp/Views/Spectrum/FrequencyBarView.swift` | **Modify** | Amber frequency text, badge, glowing LED |
| `SDRapp/SDRapp/Views/Spectrum/SpectrumContainerView.swift` | **Modify** | Dark background, remove chrome |
| `SDRapp/SDRapp/Metal/Spectrum.metal` | **Modify** | Change spectrum line/fill color from blue to amber |

---

## Task 1: Theme Constants

**Files:**
- Create: `SDRapp/SDRapp/Theme.swift`

- [ ] **Schritt 1: Datei anlegen**

```swift
// SDRapp/SDRapp/Theme.swift
import SwiftUI

enum Theme {
    // Backgrounds
    static let bgDeep    = Color(red: 0.047, green: 0.055, blue: 0.039) // #0c0e0a
    static let bgSurface = Color(red: 0.051, green: 0.059, blue: 0.043) // #0d0f0b
    static let bgRaised  = Color(red: 0.078, green: 0.094, blue: 0.063) // #141810
    static let bgSubtle  = Color(red: 0.039, green: 0.047, blue: 0.031) // #0a0c08

    // Accents
    static let amber     = Color(red: 0.784, green: 0.722, blue: 0.290) // #c8b84a
    static let olive     = Color(red: 0.416, green: 0.478, blue: 0.227) // #6a7a3a
    static let oliveMuted = Color(red: 0.290, green: 0.353, blue: 0.165) // #4a5a2a

    // Borders
    static let border    = Color(red: 0.145, green: 0.165, blue: 0.102) // #252a1a
    static let borderSubtle = Color(red: 0.102, green: 0.118, blue: 0.071) // #1a1e12

    // Status
    static let statusActive = Color(red: 0.659, green: 0.847, blue: 0.251) // #a8d840

    // Font
    static func mono(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        .system(size: size, weight: weight, design: .monospaced)
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
cd /Users/jarodschilke/Documents/Projekte/SDRapp/SDRapp
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Theme.swift
git commit -m "feat(ui): add Military Amber theme constants"
```

---

## Task 2: App-Erscheinungsbild erzwingen

**Files:**
- Modify: `SDRapp/SDRapp/SDRappApp.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/SDRappApp.swift
import SwiftUI

@main
struct SDRappApp: App {
    @State private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(appState)
                .frame(minWidth: 900, minHeight: 600)
                .preferredColorScheme(.dark)
        }
        .windowStyle(.titleBar)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/SDRappApp.swift
git commit -m "feat(ui): force dark color scheme app-wide"
```

---

## Task 3: ContentView — NavigationSplitView durch HSplitView ersetzen

**Files:**
- Modify: `SDRapp/SDRapp/ContentView.swift`

`NavigationSplitView` fügt macOS-Standard-Chrome hinzu das sich nicht vollständig überschreiben lässt. Wir ersetzen es mit `HSplitView` für volle Kontrolle.

- [ ] **Schritt 1: ContentView ersetzen**

```swift
// SDRapp/SDRapp/ContentView.swift
import SwiftUI

struct ContentView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HSplitView {
            SidebarView()
                .frame(minWidth: 200, idealWidth: 220, maxWidth: 260)
                .background(Theme.bgSurface)
            SpectrumContainerView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Theme.bgDeep)
        .onAppear {
            appState.refreshDevices()
        }
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/ContentView.swift
git commit -m "feat(ui): replace NavigationSplitView with HSplitView for custom chrome"
```

---

## Task 4: Spectrum.metal — Farbe auf Amber ändern

**Files:**
- Modify: `SDRapp/SDRapp/Metal/Spectrum.metal`

- [ ] **Schritt 1: Blau durch Amber ersetzen**

In `Spectrum.metal` die zwei Stellen mit `float4(0.25, 0.65, 1.0, ...)` auf Amber `float4(0.784, 0.722, 0.290, ...)` ändern:

```metal
// Spectrum.metal — vollständige Datei
#include <metal_stdlib>
using namespace metal;

struct SpectrumVertex {
    float4 position [[position]];
    float4 color;
};

vertex SpectrumVertex spectrum_vertex(
    uint vid [[vertex_id]],
    constant float* fftData [[buffer(0)]],
    constant uint& count    [[buffer(1)]]
) {
    float x = float(vid) / float(count - 1) * 2.0 - 1.0;
    float normalized = (fftData[vid] + 120.0) / 120.0;
    float y = normalized * 1.8 - 0.9;
    float4 color = float4(0.784, 0.722, 0.290, 1.0); // amber #c8b84a
    return { float4(x, y, 0.0, 1.0), color };
}

fragment float4 spectrum_fragment(SpectrumVertex in [[stage_in]]) {
    return in.color;
}

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
    float alpha = (vid % 2 == 0) ? 0.45 : 0.0;
    return { float4(x, y, 0.0, 1.0), float4(0.784, 0.722, 0.290, alpha) }; // amber
}
```

- [ ] **Schritt 2: SpectrumRenderer clearColor auf Amber-Dunkel**

In `SpectrumRenderer.swift` Zeile 22 ändern:

```swift
mtkView.clearColor = MTLClearColor(red: 0.031, green: 0.039, blue: 0.024, alpha: 1) // #080a06
```

- [ ] **Schritt 3: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 4: Commit**

```bash
git add SDRapp/SDRapp/Metal/Spectrum.metal SDRapp/SDRapp/Views/Spectrum/SpectrumRenderer.swift
git commit -m "feat(metal): change spectrum color from blue to amber"
```

---

## Task 5: FrequencyBarView — Amber Styling

**Files:**
- Modify: `SDRapp/SDRapp/Views/Spectrum/FrequencyBarView.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/Views/Spectrum/FrequencyBarView.swift
import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState
    @State private var inputText: String = ""
    @State private var isEditing: Bool = false

    var body: some View {
        HStack(spacing: 14) {
            if isEditing {
                TextField("MHz", text: $inputText)
                    .font(Theme.mono(20, weight: .bold))
                    .foregroundStyle(Theme.amber)
                    .textFieldStyle(.plain)
                    .onSubmit { commitFrequency() }
                    .onExitCommand { isEditing = false }
            } else {
                HStack(alignment: .lastTextBaseline, spacing: 4) {
                    Text(formatMHz(appState.frequencyHz))
                        .font(Theme.mono(20, weight: .bold))
                        .foregroundStyle(Theme.amber)
                        .tracking(2)
                    Text("MHz")
                        .font(Theme.mono(12))
                        .foregroundStyle(Theme.olive)
                }
                .onTapGesture {
                    inputText = String(format: "%.4f", Double(appState.frequencyHz) / 1e6)
                    isEditing = true
                }
            }

            Spacer()

            Text(appState.demodMode == .wbfm ? "WBFM" : "AM")
                .font(Theme.mono(10, weight: .bold))
                .tracking(1)
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(Theme.amber.opacity(0.13))
                .foregroundStyle(Theme.amber)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.amber.opacity(0.4), lineWidth: 1)
                )
                .cornerRadius(2)

            Circle()
                .fill(appState.isRunning ? Theme.statusActive : Theme.oliveMuted)
                .frame(width: 8, height: 8)
                .shadow(color: appState.isRunning ? Theme.statusActive.opacity(0.6) : .clear,
                        radius: 4)
        }
        .padding(.horizontal, 16)
        .frame(height: 44)
        .background(Theme.bgSubtle)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(Theme.borderSubtle)
                .frame(height: 1)
        }
    }

    private func commitFrequency() {
        if let mhz = Double(inputText.replacingOccurrences(of: ",", with: ".")) {
            let hz = UInt64(mhz * 1_000_000)
            if hz >= 1_000 && hz <= 6_000_000_000 {
                appState.tuneFrequency(hz)
            }
        }
        isEditing = false
    }

    private func formatMHz(_ hz: UInt64) -> String {
        String(format: "%.4f", Double(hz) / 1_000_000.0)
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Views/Spectrum/FrequencyBarView.swift
git commit -m "feat(ui): amber frequency bar styling"
```

---

## Task 6: SpectrumContainerView — Hintergrund

**Files:**
- Modify: `SDRapp/SDRapp/Views/Spectrum/SpectrumContainerView.swift`

- [ ] **Schritt 1: Hintergrundfarbe auf Theme aktualisieren**

```swift
// SDRapp/SDRapp/Views/Spectrum/SpectrumContainerView.swift
import SwiftUI
import MetalKit

struct SpectrumContainerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(spacing: 0) {
            FrequencyBarView()
                .frame(height: 44)
            SpectrumMetalView(fftData: appState.fftData)
                .frame(maxWidth: .infinity, maxHeight: 180)
                .overlay(alignment: .bottom) {
                    Rectangle()
                        .fill(Theme.borderSubtle)
                        .frame(height: 1)
                }
            WaterfallMetalView(fftData: appState.fftData)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Theme.bgDeep)
    }
}

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

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Views/Spectrum/SpectrumContainerView.swift
git commit -m "feat(ui): dark spectrum container with amber theme"
```

---

## Task 7: ModePickerView — Custom Amber Toggle

**Files:**
- Modify: `SDRapp/SDRapp/Views/Sidebar/ModePickerView.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/Views/Sidebar/ModePickerView.swift
import SwiftUI

struct ModePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HStack(spacing: 4) {
            modeButton(.wbfm, label: "WBFM")
            modeButton(.am,   label: "AM")
        }
    }

    @ViewBuilder
    private func modeButton(_ mode: DemodMode, label: String) -> some View {
        let active = appState.demodMode == mode
        Button(label) { appState.changeDemod(mode) }
            .buttonStyle(.plain)
            .font(Theme.mono(10, weight: .bold))
            .tracking(1)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 6)
            .background(active ? Theme.amber : Theme.bgRaised)
            .foregroundStyle(active ? Theme.bgDeep : Theme.olive)
            .cornerRadius(2)
            .overlay(
                RoundedRectangle(cornerRadius: 2)
                    .stroke(active ? Theme.amber : Theme.border, lineWidth: 1)
            )
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Views/Sidebar/ModePickerView.swift
git commit -m "feat(ui): custom amber mode toggle"
```

---

## Task 8: GainControlView — Custom Amber Slider

**Files:**
- Modify: `SDRapp/SDRapp/Views/Sidebar/GainControlView.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/Views/Sidebar/GainControlView.swift
import SwiftUI

struct GainControlView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("0 dB")
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
                Spacer()
                Text("\(Int(appState.gainDb)) dB")
                    .font(Theme.mono(10, weight: .bold))
                    .foregroundStyle(Theme.amber)
                Spacer()
                Text("60 dB")
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
            }
            Slider(
                value: Binding(
                    get: { appState.gainDb },
                    set: { appState.changeGain($0) }
                ),
                in: 0...60,
                step: 1
            )
            .tint(Theme.amber)
        }
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Views/Sidebar/GainControlView.swift
git commit -m "feat(ui): amber gain slider"
```

---

## Task 9: DevicePickerView — Amber Styling

**Files:**
- Modify: `SDRapp/SDRapp/Views/Sidebar/DevicePickerView.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/Views/Sidebar/DevicePickerView.swift
import SwiftUI

struct DevicePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if appState.availableDevices.isEmpty {
                Text("KEIN GERÄT")
                    .font(Theme.mono(10))
                    .foregroundStyle(Theme.olive)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 6)
                    .background(Theme.bgRaised)
                    .cornerRadius(2)
                    .overlay(
                        RoundedRectangle(cornerRadius: 2)
                            .stroke(Theme.border, lineWidth: 1)
                    )
            } else {
                Picker("", selection: Binding(
                    get: { appState.selectedDeviceArgs },
                    set: { appState.selectedDeviceArgs = $0 }
                )) {
                    Text("AUSWÄHLEN…").tag(String?.none)
                    ForEach(appState.availableDevices) { device in
                        Text(device.label).tag(Optional(device.args))
                    }
                }
                .labelsHidden()
                .pickerStyle(.menu)
                .font(Theme.mono(10))
                .tint(Theme.amber)
                .frame(maxWidth: .infinity)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Theme.bgRaised)
                .cornerRadius(2)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.border, lineWidth: 1)
                )
            }

            Button("⟳  AKTUALISIEREN") { appState.refreshDevices() }
                .buttonStyle(.plain)
                .font(Theme.mono(9, weight: .bold))
                .tracking(1)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 5)
                .foregroundStyle(Theme.oliveMuted)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.border, lineWidth: 1)
                )
                .cornerRadius(2)
        }
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: Commit**

```bash
git add SDRapp/SDRapp/Views/Sidebar/DevicePickerView.swift
git commit -m "feat(ui): amber device picker"
```

---

## Task 10: SidebarView — Vollständiges Custom Layout

**Files:**
- Modify: `SDRapp/SDRapp/Views/Sidebar/SidebarView.swift`

- [ ] **Schritt 1: Datei ersetzen**

```swift
// SDRapp/SDRapp/Views/Sidebar/SidebarView.swift
import SwiftUI

struct SidebarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Header
            HStack {
                Text("SDRapp")
                    .font(Theme.mono(11, weight: .bold))
                    .foregroundStyle(Theme.olive)
                    .tracking(2)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .frame(maxWidth: .infinity, alignment: .leading)

            divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    section("GERÄT") {
                        DevicePickerView()
                    }

                    divider()

                    section("MODUS") {
                        ModePickerView()
                    }

                    divider()

                    section("GAIN") {
                        GainControlView()
                    }

                    divider()

                    section("STEUERUNG") {
                        startStopButton()
                    }
                }
                .padding(16)
            }

            Spacer()

            // Status footer
            divider()
            HStack(spacing: 8) {
                Circle()
                    .fill(statusColor())
                    .frame(width: 6, height: 6)
                    .shadow(color: statusColor().opacity(0.7), radius: 3)
                Text(statusLabel())
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
                    .tracking(1)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        .background(Theme.bgSurface)
        .overlay(alignment: .trailing) {
            Rectangle()
                .fill(Theme.border)
                .frame(width: 1)
        }
    }

    @ViewBuilder
    private func section<Content: View>(_ label: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("◈ \(label)")
                .font(Theme.mono(9, weight: .bold))
                .foregroundStyle(Theme.oliveMuted)
                .tracking(2)
            content()
        }
    }

    @ViewBuilder
    private func divider() -> some View {
        Rectangle()
            .fill(Theme.borderSubtle)
            .frame(height: 1)
            .frame(maxWidth: .infinity)
    }

    @ViewBuilder
    private func startStopButton() -> some View {
        let running = appState.isRunning
        Button(running ? "■  STOPP" : "▶  START") {
            if running { appState.stopReceiving() } else { appState.startReceiving() }
        }
        .buttonStyle(.plain)
        .font(Theme.mono(11, weight: .bold))
        .tracking(3)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 9)
        .foregroundStyle(running ? Theme.bgDeep : Theme.amber)
        .background(running ? Theme.amber : Color.clear)
        .cornerRadius(2)
        .overlay(
            RoundedRectangle(cornerRadius: 2)
                .stroke(Theme.amber, lineWidth: 1)
        )
        .disabled(appState.selectedDeviceArgs == nil && !running)
    }

    private func statusColor() -> Color {
        if appState.isRunning { return Theme.statusActive }
        if appState.selectedDeviceArgs != nil { return Theme.olive }
        return Theme.oliveMuted
    }

    private func statusLabel() -> String {
        if appState.isRunning { return "EMPFANG" }
        if appState.selectedDeviceArgs != nil { return "BEREIT" }
        return "KEIN GERÄT"
    }
}
```

- [ ] **Schritt 2: Build prüfen**

```bash
xcodebuild -scheme SDRapp -configuration Debug CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO 2>&1 | grep -E "error:|BUILD"
```

Erwartet: `** BUILD SUCCEEDED **`

- [ ] **Schritt 3: App starten und visuell prüfen**

```bash
open /Users/jarodschilke/Library/Developer/Xcode/DerivedData/SDRapp-ajcxzpywvmxoblfhbyrgpopitzwx/Build/Products/Debug/SDRapp.app
```

Prüfen:
- Sidebar ist dunkel mit Amber-Akzenten
- Abschnitte haben `◈ LABEL`-Header
- Start-Button hat Amber-Rahmen
- Status-LED unten links
- Kein macOS-Standard-Chrome

- [ ] **Schritt 4: Commit**

```bash
git add SDRapp/SDRapp/Views/Sidebar/SidebarView.swift
git commit -m "feat(ui): full Military Amber sidebar layout"
```

---

## Task 11: Push & Abschluss

- [ ] **Schritt 1: Alle Commits pushen**

```bash
git push
```

- [ ] **Schritt 2: Finale visuelle Prüfung**

App starten und vollständig durchgehen:
- Frequenz antippen → TextField in Amber
- Gerät auswählen → Picker styled
- WBFM / AM umschalten → Amber-Toggle
- Gain schieben → Amber-Slider
- Start → Button füllt sich mit Amber, Status wechselt auf EMPFANG
- Spektrum → Amber-Linie auf dunklem Hintergrund
- Wasserfall → Viridis auf dunklem Hintergrund

---

## Self-Review

**Spec coverage:**
- ✅ `#0c0e0a` background → Theme.bgDeep (Task 1)
- ✅ `#c8b84a` amber accent → Theme.amber (Task 1)
- ✅ SF Mono durchgängig → Theme.mono() (Task 1)
- ✅ Dark appearance → Task 2
- ✅ NavigationSplitView entfernt → Task 3
- ✅ Spectrum amber → Task 4
- ✅ FrequencyBarView → Task 5
- ✅ SpectrumContainerView → Task 6
- ✅ ModePickerView → Task 7
- ✅ GainControlView → Task 8
- ✅ DevicePickerView → Task 9
- ✅ SidebarView → Task 10
- ✅ `◈` Section-Headers → Task 10
- ✅ Status-LED mit glow → Task 5 (FrequencyBarView) + Task 10 (Sidebar footer)
- ✅ Start/Stop button outline/filled → Task 10

**Placeholder scan:** Keine TBDs oder offenen Punkte.

**Type consistency:** `Theme.mono()`, `Theme.amber`, `Theme.bgDeep` etc. konsistent in allen Tasks. `appState.changeDemod()`, `appState.changeGain()`, `appState.tuneFrequency()` entsprechen der AppState-API aus dem bestehenden Code.
