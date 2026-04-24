import SwiftUI

struct SidebarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        List {
            Section("Gerät") {
                DevicePickerView()
            }
            Section("Empfang") {
                ModePickerView()
                GainControlView()
            }
            Section("Steuerung") {
                Button(appState.isRunning ? "Stopp" : "Start") {
                    if appState.isRunning {
                        appState.stopReceiving()
                    } else {
                        appState.startReceiving()
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(appState.isRunning ? .red : .blue)
                .disabled(appState.selectedDeviceArgs == nil)
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("SDRapp")
    }
}
