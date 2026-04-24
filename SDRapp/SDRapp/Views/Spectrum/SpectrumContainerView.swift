import SwiftUI
import MetalKit
import AppKit

private let SAMPLE_RATE: Double = 2_048_000

struct SpectrumContainerView: View {
    @Environment(AppState.self) var appState
    @State private var hoverX: CGFloat? = nil  // normalisierte Position (0–1) für Cursor-Linie

    var body: some View {
        VStack(spacing: 0) {
            FrequencyBarView()
                .frame(height: 44)

            ZStack {
                VStack(spacing: 0) {
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

                // Transparente Interaktions-Schicht über dem gesamten Spektrum+Wasserfall
                SpectrumInteractionOverlay(
                    onTap: { normX in
                        let offset = normX * SAMPLE_RATE - SAMPLE_RATE / 2
                        let newHz = Int64(appState.frequencyHz) + Int64(offset)
                        if newHz > 0 { appState.tuneFrequency(UInt64(newHz)) }
                    },
                    onScroll: { deltaX in
                        // 1 Scroll-Einheit ≈ 50 kHz
                        let hz = Int64(deltaX * 50_000)
                        let newHz = Int64(appState.frequencyHz) - hz
                        if newHz > 0 { appState.tuneFrequency(UInt64(newHz)) }
                    },
                    onHover: { normX in
                        hoverX = normX
                    }
                )

                // Cursor-Linie
                if let x = hoverX {
                    GeometryReader { geo in
                        let xPos = x * geo.size.width
                        Rectangle()
                            .fill(Theme.amber.opacity(0.5))
                            .frame(width: 1)
                            .offset(x: xPos)

                        // Frequenz-Label am Cursor
                        let offset = x * SAMPLE_RATE - SAMPLE_RATE / 2
                        let hoverHz = Int64(appState.frequencyHz) + Int64(offset)
                        if hoverHz > 0 {
                            Text(formatMHz(UInt64(hoverHz)))
                                .font(Theme.mono(9))
                                .foregroundStyle(Theme.amber)
                                .padding(.horizontal, 3)
                                .background(Theme.bgDeep.opacity(0.8))
                                .offset(x: min(xPos + 4, geo.size.width - 80), y: 4)
                        }
                    }
                    .allowsHitTesting(false)
                }
            }
        }
        .background(Theme.bgDeep)
    }

    private func formatMHz(_ hz: UInt64) -> String {
        String(format: "%.3f MHz", Double(hz) / 1_000_000.0)
    }
}

// MARK: - Interaktions-Overlay

struct SpectrumInteractionOverlay: NSViewRepresentable {
    var onTap: (Double) -> Void
    var onScroll: (Double) -> Void
    var onHover: (CGFloat?) -> Void

    func makeNSView(context: Context) -> InteractionNSView {
        let v = InteractionNSView()
        v.onTap = onTap
        v.onScroll = onScroll
        v.onHover = onHover
        return v
    }

    func updateNSView(_ nsView: InteractionNSView, context: Context) {
        nsView.onTap = onTap
        nsView.onScroll = onScroll
        nsView.onHover = onHover
    }
}

final class InteractionNSView: NSView {
    var onTap: ((Double) -> Void)?
    var onScroll: ((Double) -> Void)?
    var onHover: ((CGFloat?) -> Void)?

    private var trackingArea: NSTrackingArea?

    override var acceptsFirstResponder: Bool { true }

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let ta = trackingArea { removeTrackingArea(ta) }
        let ta = NSTrackingArea(
            rect: bounds,
            options: [.activeInKeyWindow, .mouseMoved, .mouseEnteredAndExited],
            owner: self, userInfo: nil
        )
        addTrackingArea(ta)
        trackingArea = ta
    }

    override func mouseDown(with event: NSEvent) {
        let loc = convert(event.locationInWindow, from: nil)
        let normX = max(0, min(1, Double(loc.x / bounds.width)))
        onTap?(normX)
    }

    override func scrollWheel(with event: NSEvent) {
        onScroll?(Double(event.scrollingDeltaX))
    }

    override func mouseMoved(with event: NSEvent) {
        let loc = convert(event.locationInWindow, from: nil)
        onHover?(loc.x / bounds.width)
    }

    override func mouseExited(with event: NSEvent) {
        onHover?(nil)
    }
}

// MARK: - Metal Views

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
