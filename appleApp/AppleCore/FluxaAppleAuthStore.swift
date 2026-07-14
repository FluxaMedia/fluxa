import Foundation
import Security

struct FluxaAppleStremioSession: Codable {
    let id: String
    let email: String
    let authKey: String
}

final class FluxaAppleAuthStore {
    private let service = "com.fluxa.app.auth"
    private let account = "stremio-session"
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    func session() -> FluxaAppleStremioSession? {
        var query = baseQuery()
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else {
            return nil
        }
        return try? decoder.decode(FluxaAppleStremioSession.self, from: data)
    }

    func save(session: FluxaAppleStremioSession) {
        guard let data = try? encoder.encode(session) else {
            return
        }
        var query = baseQuery()
        let attributes: [String: Any] = [kSecValueData as String: data]

        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecItemNotFound {
            query[kSecValueData as String] = data
            query[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlock
            SecItemAdd(query as CFDictionary, nil)
        }
    }

    func clear() {
        SecItemDelete(baseQuery() as CFDictionary)
    }

    private func baseQuery() -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account
        ]
    }
}
