import SwiftUI

struct ModePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        Picker("Modus", selection: Binding(
            get: { appState.demodMode },
            set: { appState.changeDemod($0) }
        )) {
            Text("WBFM").tag(DemodMode.wbfm)
            Text("AM").tag(DemodMode.am)
        }
        .pickerStyle(.segmented)
    }
}
