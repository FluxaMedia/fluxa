import FluxaShared
import Foundation

@MainActor
final class FluxaAppleAuthStartup {
    private let authClient: FluxaAppleStremioAuthClient
    private let authStore: FluxaAppleAuthStore

    init(authClient: FluxaAppleStremioAuthClient = FluxaAppleStremioAuthClient(), authStore: FluxaAppleAuthStore = FluxaAppleAuthStore()) {
        self.authClient = authClient
        self.authStore = authStore
    }

    func submit(request: AppleAuthSubmitSnapshot) async {
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
        FluxaApple.shared.updateAuth(snapshot: AppleAuthSnapshot(isSubmitting: isSubmitting, isAuthenticated: isAuthenticated, globalError: globalError))
    }
}
