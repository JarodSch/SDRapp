import SwiftUI
import MetalKit

// Stub — wird in Task 13 durch vollständigen Wasserfall-Renderer ersetzt
struct WaterfallMetalView: NSViewRepresentable {
    var fftData: [Float]

    func makeNSView(context: Context) -> MTKView {
        let view = MTKView()
        view.clearColor = MTLClearColor(red: 0.05, green: 0.05, blue: 0.08, alpha: 1)
        view.isPaused = true
        view.enableSetNeedsDisplay = false
        return view
    }

    func updateNSView(_ view: MTKView, context: Context) {}
    func makeCoordinator() -> Coordinator { Coordinator() }
    final class Coordinator {}
}
