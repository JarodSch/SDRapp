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
