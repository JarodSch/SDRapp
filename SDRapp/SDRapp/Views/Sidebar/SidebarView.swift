import SwiftUI

struct SidebarView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Platz für Traffic Lights (transparente Titelleiste)
            Color.clear.frame(height: 28)

            // Header
            HStack {
                Text("SDRapp")
                    .font(Theme.mono(11, weight: .bold))
                    .foregroundStyle(Theme.olive)
                    .tracking(2)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .frame(maxWidth: .infinity, alignment: .leading)

            divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    section("GERÄT") {
                        DevicePickerView()
                    }

                    divider()

                    section("MODUS") {
                        ModePickerView()
                    }

                    divider()

                    section("GAIN") {
                        GainControlView()
                    }

                    divider()

                    section("STEUERUNG") {
                        startStopButton()
                    }
                }
                .padding(16)
            }

            Spacer()

            // Status footer
            divider()
            HStack(spacing: 8) {
                Circle()
                    .fill(statusColor())
                    .frame(width: 6, height: 6)
                    .shadow(color: statusColor().opacity(0.7), radius: 3)
                Text(statusLabel())
                    .font(Theme.mono(9))
                    .foregroundStyle(Theme.olive)
                    .tracking(1)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        .background(Theme.bgSurface)
        .overlay(alignment: .trailing) {
            Rectangle()
                .fill(Theme.border)
                .frame(width: 1)
        }
    }

    @ViewBuilder
    private func section<Content: View>(_ label: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("◈ \(label)")
                .font(Theme.mono(9, weight: .bold))
                .foregroundStyle(Theme.oliveMuted)
                .tracking(2)
            content()
        }
    }

    @ViewBuilder
    private func divider() -> some View {
        Rectangle()
            .fill(Theme.borderSubtle)
            .frame(height: 1)
            .frame(maxWidth: .infinity)
    }

    @ViewBuilder
    private func startStopButton() -> some View {
        let running = appState.isRunning
        Button(running ? "■  STOPP" : "▶  START") {
            if running { appState.stopReceiving() } else { appState.startReceiving() }
        }
        .buttonStyle(.plain)
        .font(Theme.mono(11, weight: .bold))
        .tracking(3)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 9)
        .foregroundStyle(running ? Theme.bgDeep : Theme.amber)
        .background(running ? Theme.amber : Color.clear)
        .cornerRadius(2)
        .overlay(
            RoundedRectangle(cornerRadius: 2)
                .stroke(Theme.amber, lineWidth: 1)
        )
        .disabled(appState.selectedDeviceArgs == nil && !running)
    }

    private func statusColor() -> Color {
        if appState.isRunning { return Theme.statusActive }
        if appState.selectedDeviceArgs != nil { return Theme.olive }
        return Theme.oliveMuted
    }

    private func statusLabel() -> String {
        if appState.isRunning { return "EMPFANG" }
        if appState.selectedDeviceArgs != nil { return "BEREIT" }
        return "KEIN GERÄT"
    }
}
