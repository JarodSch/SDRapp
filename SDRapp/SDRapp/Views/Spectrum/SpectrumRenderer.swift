import MetalKit

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
        mtkView.clearColor = MTLClearColor(red: 0.031, green: 0.039, blue: 0.024, alpha: 1) // #080a06
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
