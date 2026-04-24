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

        // Textur (1024×512, ringförmig beschrieben)
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
