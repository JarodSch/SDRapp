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
        NavigationSplitView {
            SidebarView()
                .navigationSplitViewColumnWidth(min: 200, ideal: 240, max: 280)
        } detail: {
            SpectrumContainerView()
        }
        .onAppear {
            appState.refreshDevices()
        }
    }
}
