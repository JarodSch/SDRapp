import SwiftUI

struct FrequencyBarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HStack {
            Text(formatFrequency(appState.frequencyHz))
                .font(.system(.title3, design: .monospaced))
                .foregroundStyle(.white)
            Spacer()
        }
        .padding(.horizontal, 12)
        .background(Color(red: 0.08, green: 0.08, blue: 0.12))
    }

    func formatFrequency(_ hz: UInt64) -> String {
        let mhz = Double(hz) / 1_000_000.0
        return String(format: "%.4f MHz", mhz)
    }
}
