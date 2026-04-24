import Foundation

struct GainElement: Identifiable {
    let id = UUID()
    let name: String
    let minDb: Double
    let maxDb: Double
    let stepDb: Double
    var currentDb: Double
}

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
    // Swift importiert `struct SdrappCore *` als OpaquePointer
    private let ptr: OpaquePointer

    init() {
        guard let p = sdrapp_create() else {
            fatalError("SDRCore: sdrapp_create() returned nil — Speichermangel?")
        }
        ptr = p
    }

    deinit {
        sdrapp_destroy(ptr)
    }

    func listDevices() -> [SDRDeviceInfo] {
        var count: UInt = 0
        guard let listPtr = sdrapp_list_devices(&count) else { return [] }
        defer { sdrapp_free_device_list(listPtr) }

        var result: [SDRDeviceInfo] = []
        guard let itemsPtr = listPtr.pointee.items else { return [] }
        for i in 0..<Int(count) {
            let item = itemsPtr[i]
            let label = item.label.map { String(cString: $0) } ?? "Unknown"
            let args  = item.args.map  { String(cString: $0) } ?? ""
            result.append(SDRDeviceInfo(label: label, args: args))
        }
        return result
    }

    func setDevice(_ args: String) {
        sdrapp_set_device(ptr, args)
    }

    func setFrequency(_ hz: UInt64) {
        sdrapp_set_frequency(ptr, hz)
    }

    func setGain(_ db: Double) {
        sdrapp_set_gain(ptr, db)
    }

    func setGainElement(_ name: String, _ db: Double) {
        sdrapp_set_gain_element(ptr, name, db)
    }

    func listGainElements() -> [GainElement] {
        var count: UInt = 0
        guard let listPtr = sdrapp_list_gains(ptr, &count) else { return [] }
        defer { sdrapp_free_gain_list(listPtr) }
        guard let itemsPtr = listPtr.pointee.items else { return [] }
        return (0..<Int(count)).map { i in
            let item = itemsPtr[i]
            return GainElement(
                name:      item.name.map { String(cString: $0) } ?? "",
                minDb:     item.min_db,
                maxDb:     item.max_db,
                stepDb:    item.step_db,
                currentDb: item.current_db
            )
        }
    }

    func setDemod(_ mode: DemodMode) {
        sdrapp_set_demod(ptr, mode.rawValue)
    }

    @discardableResult
    func start() -> Bool {
        sdrapp_start(ptr)
    }

    func stop() {
        sdrapp_stop(ptr)
    }

    func getFFT(size: Int = 1024) -> [Float] {
        var buf = [Float](repeating: -120, count: size)
        sdrapp_get_fft(ptr, &buf, UInt(size))
        return buf
    }
}
