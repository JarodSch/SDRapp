import AppKit
import SwiftUI

/// Macht die macOS-Titelleiste transparent und erweitert den Fensterinhalt darunter.
struct WindowConfigurator: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        DispatchQueue.main.async {
            guard let window = view.window else { return }
            window.titlebarAppearsTransparent = true
            window.titleVisibility = .hidden
            window.styleMask.insert(.fullSizeContentView)
            window.backgroundColor = NSColor(
                red: 0.039, green: 0.047, blue: 0.031, alpha: 1.0
            )
        }
        return view
    }
    func updateNSView(_ nsView: NSView, context: Context) {}
}
