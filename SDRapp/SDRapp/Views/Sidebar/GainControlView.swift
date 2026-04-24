import SwiftUI

struct GainControlView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Gain")
                Spacer()
                Text("\(Int(appState.gainDb)) dB")
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }
            Slider(
                value: Binding(
                    get: { appState.gainDb },
                    set: { appState.changeGain($0) }
                ),
                in: 0...60,
                step: 1
            )
        }
    }
}
