import SwiftUI

struct GainControlView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("0 dB")
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
                Spacer()
                Text("\(Int(appState.gainDb)) dB")
                    .font(Theme.mono(10, weight: .bold))
                    .foregroundStyle(Theme.amber)
                Spacer()
                Text("60 dB")
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
            }
            Slider(
                value: Binding(
                    get: { appState.gainDb },
                    set: { appState.changeGain($0) }
                ),
                in: 0...60,
                step: 1
            )
            .tint(Theme.amber)
        }
    }
}
