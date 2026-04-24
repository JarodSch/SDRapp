// SDRapp/SDRapp/Theme.swift
import SwiftUI

enum Theme {
    // Backgrounds
    static let bgDeep    = Color(red: 0.047, green: 0.055, blue: 0.039) // #0c0e0a
    static let bgSurface = Color(red: 0.051, green: 0.059, blue: 0.043) // #0d0f0b
    static let bgRaised  = Color(red: 0.078, green: 0.094, blue: 0.063) // #141810
    static let bgSubtle  = Color(red: 0.039, green: 0.047, blue: 0.031) // #0a0c08

    // Accents
    static let amber     = Color(red: 0.784, green: 0.722, blue: 0.290) // #c8b84a
    static let olive     = Color(red: 0.416, green: 0.478, blue: 0.227) // #6a7a3a
    static let oliveMuted = Color(red: 0.290, green: 0.353, blue: 0.165) // #4a5a2a

    // Borders
    static let border    = Color(red: 0.145, green: 0.165, blue: 0.102) // #252a1a
    static let borderSubtle = Color(red: 0.102, green: 0.118, blue: 0.071) // #1a1e12

    // Status
    static let statusActive = Color(red: 0.659, green: 0.847, blue: 0.251) // #a8d840

    // Font
    static func mono(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        .system(size: size, weight: weight, design: .monospaced)
    }
}
