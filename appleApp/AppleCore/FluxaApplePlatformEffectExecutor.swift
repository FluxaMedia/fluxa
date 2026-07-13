import Foundation

protocol FluxaApplePlatformEffectHandler: AnyObject {
    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue
}

final class FluxaApplePlatformEffectExecutor: FluxaAppleHeadlessEffectExecutor {
    private let handler: any FluxaApplePlatformEffectHandler

    init(handler: any FluxaApplePlatformEffectHandler) {
        self.handler = handler
    }

    func execute(effect: FluxaAppleHeadlessEffect) async -> FluxaAppleHeadlessCompletion {
        do {
            return .success(effectId: effect.id, value: try await handler.execute(effect: effect))
        } catch {
            let code = (error as NSError).domain
            return .failure(effectId: effect.id, code: code)
        }
    }
}
