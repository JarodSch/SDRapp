import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState
    @State private var inputText: String = ""
    @State private var isEditing: Bool = false

    var body: some View {
        HStack(spacing: 14) {
            if isEditing {
                TextField("MHz", text: $inputText)
                    .font(Theme.mono(20, weight: .bold))
                    .foregroundStyle(Theme.amber)
                    .textFieldStyle(.plain)
                    .onSubmit { commitFrequency() }
                    .onExitCommand { isEditing = false }
            } else {
                HStack(alignment: .lastTextBaseline, spacing: 4) {
                    Text(formatMHz(appState.frequencyHz))
                        .font(Theme.mono(20, weight: .bold))
                        .foregroundStyle(Theme.amber)
                        .tracking(2)
                    Text("MHz")
                        .font(Theme.mono(12))
                        .foregroundStyle(Theme.olive)
                }
                .onTapGesture {
                    inputText = String(format: "%.4f", Double(appState.frequencyHz) / 1e6)
                    isEditing = true
                }
            }

            Spacer()

            Text(appState.demodMode == .wbfm ? "WBFM" : "AM")
                .font(Theme.mono(10, weight: .bold))
                .tracking(1)
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(Theme.amber.opacity(0.13))
                .foregroundStyle(Theme.amber)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.amber.opacity(0.4), lineWidth: 1)
                )
                .cornerRadius(2)

            Circle()
                .fill(appState.isRunning ? Theme.statusActive : Theme.oliveMuted)
                .frame(width: 8, height: 8)
                .shadow(color: appState.isRunning ? Theme.statusActive.opacity(0.6) : .clear,
                        radius: 4)
        }
        .padding(.horizontal, 16)
        .frame(height: 44)
        .background(Theme.bgSubtle)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(Theme.borderSubtle)
                .frame(height: 1)
        }
    }

    private func commitFrequency() {
        if let mhz = Double(inputText.replacingOccurrences(of: ",", with: ".")),
           mhz > 0, mhz <= 6000 {
            appState.tuneFrequency(UInt64(mhz * 1_000_000))
        }
        isEditing = false
    }

    private func formatMHz(_ hz: UInt64) -> String {
        String(format: "%.4f", Double(hz) / 1_000_000.0)
    }
}
