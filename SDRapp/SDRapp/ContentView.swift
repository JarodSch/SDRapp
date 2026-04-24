//
//  ContentView.swift
//  SDRapp
//
//  Created by Jarod Schilke on 24.04.26.
//

import SwiftUI

struct ContentView: View {
    @Environment(AppState.self) var appState

    var body: some View {
        HSplitView {
            SidebarView()
                .frame(minWidth: 200, idealWidth: 220, maxWidth: 260)
                .background(Theme.bgSurface)
            SpectrumContainerView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .ignoresSafeArea()
        .background(WindowConfigurator())
        .background(Theme.bgDeep)
        .onAppear {
            appState.refreshDevices()
        }
    }
}
