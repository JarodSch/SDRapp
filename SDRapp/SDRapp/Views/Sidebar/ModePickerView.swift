import SwiftUI

struct ModePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HStack(spacing: 4) {
            modeButton(.wbfm, label: "WBFM")
            modeButton(.am,   label: "AM")
        }
    }

    @ViewBuilder
    private func modeButton(_ mode: DemodMode, label: String) -> some View {
        let active = appState.demodMode == mode
        Button { appState.changeDemod(mode) } label: {
            Text(label)
                .font(Theme.mono(10, weight: .bold))
                .tracking(1)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 6)
                .background(active ? Theme.amber : Theme.bgRaised)
                .foregroundStyle(active ? Theme.bgDeep : Theme.olive)
                .cornerRadius(2)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(active ? Theme.amber : Theme.border, lineWidth: 1)
                )
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
