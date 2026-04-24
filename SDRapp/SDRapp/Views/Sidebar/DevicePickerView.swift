import SwiftUI

struct DevicePickerView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if appState.availableDevices.isEmpty {
                Text("KEIN GERÄT")
                    .font(Theme.mono(10))
                    .foregroundStyle(Theme.olive)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 6)
                    .background(Theme.bgRaised)
                    .cornerRadius(2)
                    .overlay(
                        RoundedRectangle(cornerRadius: 2)
                            .stroke(Theme.border, lineWidth: 1)
                    )
            } else {
                Picker("", selection: Binding(
                    get: { appState.selectedDeviceArgs },
                    set: { appState.selectedDeviceArgs = $0 }
                )) {
                    Text("AUSWÄHLEN…").tag(String?.none)
                    ForEach(appState.availableDevices) { device in
                        Text(device.label).tag(Optional(device.args))
                    }
                }
                .labelsHidden()
                .pickerStyle(.menu)
                .font(Theme.mono(10))
                .tint(Theme.amber)
                .frame(maxWidth: .infinity)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Theme.bgRaised)
                .cornerRadius(2)
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.border, lineWidth: 1)
                )
            }

            Button("⟳  AKTUALISIEREN") { appState.refreshDevices() }
                .buttonStyle(.plain)
                .font(Theme.mono(9, weight: .bold))
                .tracking(1)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 5)
                .foregroundStyle(Theme.oliveMuted)
                .background(Color.clear)
                .contentShape(Rectangle())
                .overlay(
                    RoundedRectangle(cornerRadius: 2)
                        .stroke(Theme.border, lineWidth: 1)
                )
        }
    }
}
