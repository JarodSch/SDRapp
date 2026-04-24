import Foundation

enum DemodMode: UInt32 {
    case am = 0
    case wbfm = 1
}

struct SDRDeviceInfo: Identifiable {
    let id = UUID()
    let label: String
    let args: String
}

final class SDRCore {
    private let ptr: UnsafeMutableRawPointer

    init() {
        ptr = UnsafeMutableRawPointer(sdrapp_create())
    }

    deinit {
        sdrapp_destroy(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func listDevices() -> [SDRDeviceInfo] {
        var count: Int = 0
        guard let listPtr = sdrapp_list_devices(&count) else { return [] }
        defer { sdrapp_free_device_list(listPtr) }

        var result: [SDRDeviceInfo] = []
        let itemsPtr = listPtr.pointee.items
        for i in 0..<count {
            let item = itemsPtr![i]
            let label = item.label.map { String(cString: $0) } ?? "Unknown"
            let args  = item.args.map  { String(cString: $0) } ?? ""
            result.append(SDRDeviceInfo(label: label, args: args))
        }
        return result
    }

    func setDevice(_ args: String) {
        sdrapp_set_device(ptr.assumingMemoryBound(to: SdrappCore.self), args)
    }

    func setFrequency(_ hz: UInt64) {
        sdrapp_set_frequency(ptr.assumingMemoryBound(to: SdrappCore.self), hz)
    }

    func setGain(_ db: Double) {
        sdrapp_set_gain(ptr.assumingMemoryBound(to: SdrappCore.self), db)
    }

    func setDemod(_ mode: DemodMode) {
        sdrapp_set_demod(ptr.assumingMemoryBound(to: SdrappCore.self), mode.rawValue)
    }

    @discardableResult
    func start() -> Bool {
        sdrapp_start(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func stop() {
        sdrapp_stop(ptr.assumingMemoryBound(to: SdrappCore.self))
    }

    func getFFT(size: Int = 1024) -> [Float] {
        var buf = [Float](repeating: -120, count: size)
        sdrapp_get_fft(ptr.assumingMemoryBound(to: SdrappCore.self), &buf, size)
        return buf
    }
}
