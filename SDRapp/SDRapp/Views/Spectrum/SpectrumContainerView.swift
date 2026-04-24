import SwiftUI

// Stub — wird in Task 12 durch Metal-Renderer ersetzt
struct SpectrumContainerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        ZStack {
            Color.black
            Text("Spektrum")
                .foregroundStyle(.secondary)
        }
    }
}
