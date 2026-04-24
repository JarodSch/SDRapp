import SwiftUI

struct GainControlView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if appState.gainElements.isEmpty {
                gainSlider(
                    label: "OVERALL",
                    value: Binding(
                        get: { appState.gainDb },
                        set: { appState.changeGain($0) }
                    ),
                    min: 0, max: 60, step: 1
                )
            } else {
                ForEach(appState.gainElements) { el in
                    gainSlider(
                        label: el.name.uppercased(),
                        value: Binding(
                            get: { el.currentDb },
                            set: { appState.changeGainElement(el.name, $0) }
                        ),
                        min: el.minDb, max: el.maxDb, step: el.stepDb
                    )
                }
            }
        }
    }

    @ViewBuilder
    private func gainSlider(label: String, value: Binding<Double>, min: Double, max: Double, step: Double) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.oliveMuted)
                Spacer()
                Text("\(Int(value.wrappedValue)) dB")
                    .font(Theme.mono(10, weight: .bold))
                    .foregroundStyle(Theme.amber)
            }
            Slider(value: value, in: min...max, step: step > 0 ? step : 1)
                .tint(Theme.amber)
        }
    }
}
