import Foundation

enum FluxaAppleStremioAuthError: Error {
    case invalidCredentials
    case network(Error)
}

struct FluxaAppleStremioAuthClient {
    private let session: URLSession
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()
    private static let baseUrl = URL(string: "https://api.strem.io/")!

    init(session: URLSession = .shared) {
        self.session = session
    }

    func login(email: String, password: String) async throws -> FluxaAppleStremioSession {
        try await authenticate(path: "api/login", email: email, password: password)
    }

    func register(email: String, password: String) async throws -> FluxaAppleStremioSession {
        try await authenticate(path: "api/register", email: email, password: password)
    }

    private func authenticate(path: String, email: String, password: String) async throws -> FluxaAppleStremioSession {
        var request = URLRequest(url: Self.baseUrl.appendingPathComponent(path))
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.setValue(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
            forHTTPHeaderField: "User-Agent"
        )
        request.httpBody = try encoder.encode(FluxaAppleStremioLoginRequest(email: email, password: password))

        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw FluxaAppleStremioAuthError.network(error)
        }

        guard let httpResponse = response as? HTTPURLResponse, (200..<300).contains(httpResponse.statusCode) else {
            throw FluxaAppleStremioAuthError.invalidCredentials
        }
        guard let decoded = try? decoder.decode(FluxaAppleStremioAuthResponse.self, from: data) else {
            throw FluxaAppleStremioAuthError.invalidCredentials
        }
        let user = decoded.result.user
        return FluxaAppleStremioSession(id: user.id, email: user.email, authKey: user.authKey)
    }
}

private struct FluxaAppleStremioLoginRequest: Encodable {
    let email: String
    let password: String
}

private struct FluxaAppleStremioAuthResponse: Decodable {
    let result: FluxaAppleStremioAuthResult
}

private struct FluxaAppleStremioAuthResult: Decodable {
    let user: FluxaAppleStremioAuthUser
}

private struct FluxaAppleStremioAuthUser: Decodable {
    let id: String
    let email: String
    let authKey: String
}
