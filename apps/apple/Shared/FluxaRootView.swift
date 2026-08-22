import FluxaShared
import SwiftUI
import UIKit

struct FluxaRootView: UIViewControllerRepresentable {
    func makeUIViewController(context: Context) -> UIViewController {
        FluxaApple.shared.rootViewController()
    }

    func updateUIViewController(_ uiViewController: UIViewController, context: Context) {}
}
