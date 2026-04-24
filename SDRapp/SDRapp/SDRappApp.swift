//
//  SDRappApp.swift
//  SDRapp
//
//  Created by Jarod Schilke on 24.04.26.
//

import SwiftUI

@main
struct SDRappApp: App {
    @State private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(appState)
                .frame(minWidth: 900, minHeight: 600)
                .preferredColorScheme(.dark)
        }
        .windowStyle(.titleBar)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
