import SwiftUI
import Observation

@Observable
final class AppState {
    // Gerät
    var availableDevices: [SDRDeviceInfo] = []
    var selectedDeviceArgs: String? = nil

    // Empfangsparameter
    var frequencyHz: UInt64 = 100_000_000   // 100 MHz
    var gainDb: Double = 30.0
    var gainElements: [GainElement] = []
    var demodMode: DemodMode = .wbfm
    var bandwidthHz: UInt64 = 200_000

    // Laufzeit-Status
    var isRunning: Bool = false
    var fftData: [Float] = Array(repeating: -120, count: 1024)

    private let core = SDRCore()
    private var fftTimer: Timer?

    func refreshDevices() {
        Task.detached { [weak self] in
            guard let self else { return }
            let devices = self.core.listDevices()
            await MainActor.run { self.availableDevices = devices }
        }
    }

    func selectDevice(_ args: String?) {
        selectedDeviceArgs = args
        gainElements = []
        if args != nil {
            core.setDevice(args!)
            refreshGainElements()
        }
    }

    func startReceiving() {
        guard let args = selectedDeviceArgs else { return }
        core.setDevice(args)
        core.setFrequency(frequencyHz)
        // Per-Element-Gains anwenden falls gesetzt, sonst Overall-Gain
        if gainElements.isEmpty {
            core.setGain(gainDb)
        } else {
            for el in gainElements { core.setGainElement(el.name, el.currentDb) }
        }
        core.setDemod(demodMode)
        isRunning = core.start()
        if isRunning { startFFTPolling() }
    }

    func stopReceiving() {
        core.stop()
        isRunning = false
        stopFFTPolling()
    }

    func tuneFrequency(_ hz: UInt64) {
        frequencyHz = hz
        core.setFrequency(hz)
    }

    func changeGain(_ db: Double) {
        gainDb = db
        core.setGain(db)
    }

    func changeGainElement(_ name: String, _ db: Double) {
        if let i = gainElements.firstIndex(where: { $0.name == name }) {
            gainElements[i].currentDb = db
        }
        core.setGainElement(name, db)
    }

    func refreshGainElements() {
        Task.detached { [weak self] in
            guard let self else { return }
            let elements = self.core.listGainElements()
            await MainActor.run { self.gainElements = elements }
        }
    }

    func changeDemod(_ mode: DemodMode) {
        demodMode = mode
        core.setDemod(mode)
    }

    private func startFFTPolling() {
        fftTimer = Timer.scheduledTimer(withTimeInterval: 1.0/30.0, repeats: true) { [weak self] _ in
            guard let self else { return }
            self.fftData = self.core.getFFT()
        }
    }

    private func stopFFTPolling() {
        fftTimer?.invalidate()
        fftTimer = nil
    }
}
