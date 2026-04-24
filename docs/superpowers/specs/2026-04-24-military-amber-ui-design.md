# SDRapp UI Redesign — Military Amber

**Date:** 2026-04-24  
**Status:** Approved

## Overview

Complete visual redesign of the SDRapp macOS interface from the default macOS light/dark appearance to a cohesive "Military Amber" aesthetic — dark background with amber/gold accents and monospace typography throughout.

## Design Language

### Colors

| Role | Value | Usage |
|------|-------|-------|
| Background (deep) | `#0c0e0a` | Main window background |
| Background (surface) | `#0d0f0b` | Sidebar |
| Background (elevated) | `#141810` | Input fields, dropdowns |
| Background (subtle) | `#0a0c08` | Titlebar, frequency bar |
| Accent (amber) | `#c8b84a` | Active controls, spectrum line, frequency text, Start button border |
| Secondary (olive) | `#6a7a3a` | Secondary labels, dB markers, frequency axis |
| Muted (dark olive) | `#4a5a2a` | Section headers, inactive elements |
| Border | `#252a1a` | Dividers, input borders |
| Border subtle | `#1a1e12` | Grid lines, separators |
| Status active | `#a8d840` | Running indicator LED (with glow) |

### Typography

- **Font:** SF Mono (system monospace fallback: Menlo, Courier New)
- **Frequency display:** 20px bold, letter-spacing 2px
- **Section headers:** 9px, weight 700, letter-spacing 2px, UPPERCASE, prefixed with `◈`
- **Labels:** 9–11px monospace
- **All text:** monospace throughout — no sans-serif anywhere

### Shape & Borders

- Border radius: 2–3px maximum (sharp, technical feel)
- Borders: 1px solid, very subtle
- No shadows, no blur effects
- Grid lines in spectrum: `#1a1e12` (barely visible)

## Layout

### Window Structure

```
┌─ Titlebar (traffic lights + "SDRapp" label) ──────────────────┐
├─ Sidebar (220px) ─┬─ Frequency Bar ────────────────────────────┤
│                   │ 100.0000 MHz          [WBFM] ●             │
│ ◈ GERÄT           ├────────────────────────────────────────────┤
│   [HackRF One ▾]  │                                            │
│   [⟳ AKTUALISIEREN]│         Spectrum (180px fixed)            │
│                   │         Amber line + fill, dB grid         │
│ ◈ MODUS           ├────────────────────────────────────────────┤
│   [WBFM] [AM]     │                                            │
│                   │         Waterfall (flex, fills rest)       │
│ ◈ GAIN            │         Viridis colormap                   │
│   ──●────  30 dB  │         Frequency axis labels at bottom    │
│                   │                                            │
│ ◈ STEUERUNG       │                                            │
│   [▶ START]       │                                            │
│                   │                                            │
│ ● BEREIT          │                                            │
└───────────────────┴────────────────────────────────────────────┘
```

### Sidebar Sections

Each section uses a `◈ LABEL` header (9px, #4a5a2a) followed by controls. Sections separated by 1px dividers (`#1a1e12`).

- **◈ GERÄT:** Dropdown-style device picker (custom styled) + Aktualisieren button
- **◈ MODUS:** Two-state toggle: WBFM / AM (amber fill for active, border-only for inactive)
- **◈ GAIN:** Custom slider with amber thumb + gradient track, dB label
- **◈ STEUERUNG:** Start/Stop button — outline style when stopped (amber border + text), solid amber fill when running shows "■ STOPP"
- Status LED at bottom: `#a8d840` dot + "BEREIT" / "EMPFANG" / "KEIN GERÄT" label

### Frequency Bar

- Background: `#0a0c08`, border-bottom: `#1a1e12`
- Frequency: 20px bold monospace, `#c8b84a`, letter-spacing 2px
- Unit "MHz" in smaller `#6a7a3a`
- Demod badge: `#c8b84a22` background, `#c8b84a` border + text
- Status LED: `#a8d840` with CSS glow (`box-shadow: 0 0 6px #a8d84088`)
- Tap-to-edit behavior preserved: click frequency → TextField appears

### Spectrum View

- Background: `#080a06`
- Grid: subtle horizontal and vertical lines in `#1a1e12`
- dB labels: left edge, `#4a5a2a`, 8px
- Spectrum line: `#c8b84a`, 1.5px stroke
- Fill: linear gradient from `#c8b84a` (60% opacity) to transparent
- Height: fixed 180px

### Waterfall View

- Background: `#080a06`
- Colormap: Viridis (unchanged — purple → blue → green → yellow)
- Frequency axis: bottom edge labels in `#4a5a2a`
- Flex height: fills remaining window space

## Component Changes

### Files to modify

| File | Change |
|------|--------|
| `SidebarView.swift` | Replace `List`/`.listStyle(.sidebar)` with custom dark `VStack`, amber section headers |
| `DevicePickerView.swift` | Style picker with amber border, dark background |
| `ModePickerView.swift` | Custom two-button toggle, amber active state |
| `GainControlView.swift` | Custom amber slider |
| `FrequencyBarView.swift` | Amber frequency text, badge, glowing LED |
| `SpectrumContainerView.swift` | Dark background, remove default chrome |
| `ContentView.swift` | Set `.preferredColorScheme(.dark)`, remove NavigationSplitView default chrome |
| `SDRappApp.swift` | Apply dark appearance at app level |

### Metal shaders — no changes needed

Spectrum and Waterfall Metal renderers are unchanged. The amber spectrum line color is set in `SpectrumRenderer.swift` (currently blue — change to `#c8b84a`).

## Behaviour — Unchanged

- Tap-to-edit frequency (FrequencyBarView)
- Start/Stop logic
- Device refresh
- All Rust DSP pipeline

## Out of Scope

- Frequency axis tick marks on spectrum (future)
- Signal strength meter (future)
- Dark/Light mode toggle (app is always dark)
