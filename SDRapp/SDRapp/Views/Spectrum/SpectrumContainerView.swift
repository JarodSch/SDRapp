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
