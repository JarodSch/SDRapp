import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState
    @State private var inputText: String = ""
    @State private var isEditing: Bool = false

    var body: some View {
        HStack(spacing: 12) {
            // Frequenz-Anzeige / Eingabe
            if isEditing {
                TextField("MHz", text: $inputText)
                    .font(.system(.title3, design: .monospaced))
                    .foregroundStyle(.white)
                    .textFieldStyle(.plain)
                    .onSubmit { commitFrequency() }
                    .onExitCommand { isEditing = false }
            } else {
                Text(formatFrequency(appState.frequencyHz))
                    .font(.system(.title3, design: .monospaced))
                    .foregroundStyle(.white)
                    .onTapGesture {
                        inputText = String(format: "%.4f", Double(appState.frequencyHz) / 1e6)
                        isEditing = true
                    }
            }

            Spacer()

            // Modus-Anzeige
            Text(appState.demodMode == .wbfm ? "WBFM" : "AM")
                .font(.caption)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color.blue.opacity(0.3))
                .cornerRadius(4)
                .foregroundStyle(.blue)

            // Status
            Circle()
                .fill(appState.isRunning ? Color.green : Color.gray)
                .frame(width: 8, height: 8)
        }
        .padding(.horizontal, 12)
        .frame(height: 44)
        .background(Color(red: 0.08, green: 0.08, blue: 0.12))
    }

    private func commitFrequency() {
        if let mhz = Double(inputText.replacingOccurrences(of: ",", with: ".")) {
            let hz = UInt64(mhz * 1_000_000)
            if hz >= 1_000 && hz <= 6_000_000_000 {  // 1 kHz bis 6 GHz
                appState.tuneFrequency(hz)
            }
        }
        isEditing = false
    }

    private func formatFrequency(_ hz: UInt64) -> String {
        String(format: "%.4f MHz", Double(hz) / 1_000_000.0)
    }
}
