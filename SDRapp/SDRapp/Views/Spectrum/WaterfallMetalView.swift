import SwiftUI
import MetalKit

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
