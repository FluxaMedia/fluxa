import FluxaShared
import Foundation

@MainActor
final class FluxaAppleAuthStartup {
    private let authClient: FluxaAppleStremioAuthClient
    private let authStore: FluxaAppleAuthStore
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    init(authClient: FluxaAppleStremioAuthClient = FluxaAppleStremioAuthClient(), authStore: FluxaAppleAuthStore = FluxaAppleAuthStore()) {
        self.authClient = authClient
        self.authStore = authStore
    }

    func submit(requestJson: String) async {
        guard let request = try? decoder.decode(FluxaAppleAuthSubmitRequest.self, from: Data(requestJson.utf8)) else {
            return
        }
        do {
            let session = request.isSignup
                ? try await authClient.register(email: request.email, password: request.password)
                : try await authClient.login(email: request.email, password: request.password)
            authStore.save(session: session)
            updateState(isSubmitting: false, isAuthenticated: true)
        } catch {
            updateState(isSubmitting: false, isAuthenticated: false, globalError: "Sign in failed. Check your email and password and try again.")
        }
    }

    private func updateState(isSubmitting: Bool, isAuthenticated: Bool, globalError: String? = nil) {
        let snapshot = FluxaAppleJsonValue.object([
            "isSubmitting": .boolean(isSubmitting),
            "isAuthenticated": .boolean(isAuthenticated),
            "globalError": globalError.map { FluxaAppleJsonValue.string($0) } ?? .null
        ])
        guard let data = try? encoder.encode(snapshot) else {
            return
        }
        FluxaApple.shared.updateAuthJson(authJson: String(decoding: data, as: UTF8.self))
    }
}

final class FluxaAppleAuthNotificationObserver {
    private let startup: FluxaAppleAuthStartup
    private let tokens: [NSObjectProtocol]

    init(startup: FluxaAppleAuthStartup) {
        self.startup = startup
        tokens = [
            NotificationCenter.default.addObserver(
                forName: Notification.Name("FluxaAppleAuthSubmitRequested"),
                object: nil,
                queue: .main
            ) { [weak startup] notification in
                guard let requestJson = notification.object as? String else {
                    return
                }
                Task { @MainActor in
                    await startup?.submit(requestJson: requestJson)
                }
            }
        ]
    }

    deinit {
        tokens.forEach { NotificationCenter.default.removeObserver($0) }
    }
}

private struct FluxaAppleAuthSubmitRequest: Decodable {
    let email: String
    let password: String
    let isSignup: Bool
}
