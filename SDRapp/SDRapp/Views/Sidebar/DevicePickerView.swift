import SwiftUI

struct DevicePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        if appState.availableDevices.isEmpty {
            Text("Kein Gerät gefunden")
                .foregroundStyle(.secondary)
                .font(.caption)
        } else {
            Picker("Gerät", selection: Binding(
                get: { appState.selectedDeviceArgs },
                set: { appState.selectedDeviceArgs = $0 }
            )) {
                Text("Auswählen…").tag(String?.none)
                ForEach(appState.availableDevices) { device in
                    Text(device.label).tag(Optional(device.args))
                }
            }
            .pickerStyle(.menu)
        }
        Button("Aktualisieren") {
            appState.refreshDevices()
        }
        .font(.caption)
    }
}
