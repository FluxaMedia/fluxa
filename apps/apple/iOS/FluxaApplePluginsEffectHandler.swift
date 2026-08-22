import Foundation

final class FluxaApplePluginsEffectHandler: FluxaApplePlatformEffectHandler {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case "fetchPluginManifest":
            return try await fetchPluginManifest(effect: effect)
        default:
            throw NSError(domain: "FluxaAppleUnsupportedEffect", code: 1)
        }
    }

    private func fetchPluginManifest(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              case .string(let manifestUrl)? = payload["manifestUrl"],
              let url = URL(string: manifestUrl) else {
            throw NSError(domain: "invalid_manifest_url", code: 1)
        }
        let (data, response) = try await session.data(from: url)
        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            let code = (response as? HTTPURLResponse)?.statusCode ?? 0
            throw NSError(domain: "http_\(code)", code: 1)
        }
        let body = String(decoding: data, as: UTF8.self)
        let resultJson = coreInvoke(method: "pluginManifestParse", argsJson: body)
        let manifest = try parseCoreInvokeEnvelope(resultJson)
        return .object([
            "manifestUrl": .string(manifestUrl),
            "manifest": manifest
        ])
    }

    private func parseCoreInvokeEnvelope(_ json: String) throws -> FluxaAppleJsonValue {
        let envelope = try JSONDecoder().decode(FluxaAppleJsonValue.self, from: Data(json.utf8))
        guard case .object(let object) = envelope else {
            throw NSError(domain: "core_invoke_parse_failed", code: 1)
        }
        guard case .boolean(true)? = object["ok"] else {
            throw NSError(domain: "pluginManifestParse_failed", code: 1)
        }
        return object["value"] ?? .null
    }
}
