import Foundation

indirect enum FluxaAppleJsonValue: Codable, Sendable {
    case null
    case boolean(Bool)
    case number(Double)
    case string(String)
    case array([FluxaAppleJsonValue])
    case object([String: FluxaAppleJsonValue])

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .boolean(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([FluxaAppleJsonValue].self) {
            self = .array(value)
        } else {
            self = .object(try container.decode([String: FluxaAppleJsonValue].self))
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .null:
            try container.encodeNil()
        case .boolean(let value):
            try container.encode(value)
        case .number(let value):
            try container.encode(value)
        case .string(let value):
            try container.encode(value)
        case .array(let value):
            try container.encode(value)
        case .object(let value):
            try container.encode(value)
        }
    }
}

struct FluxaAppleHeadlessEffect: Decodable, Sendable {
    let id: String
    let type: String
    let generation: UInt64
    let payload: FluxaAppleJsonValue
}

struct FluxaAppleHeadlessCompletion: Encodable, Sendable {
    let effectId: String
    let status: String
    let value: FluxaAppleJsonValue
    let error: FluxaAppleJsonValue

    static func success(effectId: String, value: FluxaAppleJsonValue = .null) -> Self {
        Self(effectId: effectId, status: "ok", value: value, error: .null)
    }

    static func failure(effectId: String, code: String) -> Self {
        Self(
            effectId: effectId,
            status: "error",
            value: .null,
            error: .object(["code": .string(code)])
        )
    }
}

struct FluxaAppleHeadlessResult: Decodable, Sendable {
    let state: [String: FluxaAppleJsonValue]
    let effects: [FluxaAppleHeadlessEffect]
}

protocol FluxaAppleHeadlessEffectExecutor: AnyObject {
    func execute(effect: FluxaAppleHeadlessEffect) async -> FluxaAppleHeadlessCompletion
}

final class FluxaAppleHeadlessCoordinator {
    private let runtime: FluxaAppleHeadlessRuntime
    private let executor: any FluxaAppleHeadlessEffectExecutor
    private let maxEffectsPerDispatch: Int
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()
    private let dispatchMutex = FluxaAppleAsyncMutex()

    init(
        runtime: FluxaAppleHeadlessRuntime,
        executor: any FluxaAppleHeadlessEffectExecutor,
        maxEffectsPerDispatch: Int = 64
    ) {
        self.runtime = runtime
        self.executor = executor
        self.maxEffectsPerDispatch = maxEffectsPerDispatch
    }

    func dispatch(actionJson: String) async throws -> FluxaAppleHeadlessResult {
        try await withExclusiveRuntime {
            try await drain(runtime.dispatch(actionJson: actionJson))
        }
    }

    func complete(completion: FluxaAppleHeadlessCompletion) async throws -> FluxaAppleHeadlessResult {
        try await withExclusiveRuntime {
            try await drain(runtime.completeEffect(resultJson: try encode(completion)))
        }
    }

    private func withExclusiveRuntime<T>(
        operation: () async throws -> T
    ) async throws -> T {
        await dispatchMutex.acquire()
        do {
            let value = try await operation()
            await dispatchMutex.release()
            return value
        } catch {
            await dispatchMutex.release()
            throw error
        }
    }

    private func drain(_ initialJson: String) async throws -> FluxaAppleHeadlessResult {
        var current = try decode(initialJson)
        var pending = current.effects
        var remaining = maxEffectsPerDispatch
        while !pending.isEmpty, remaining > 0 {
            let effect = pending.removeFirst()
            let completion = await executor.execute(effect: effect)
            current = try decode(runtime.completeEffect(resultJson: try encode(completion)))
            pending.append(contentsOf: current.effects)
            remaining -= 1
        }
        return current
    }

    private func decode(_ json: String) throws -> FluxaAppleHeadlessResult {
        try decoder.decode(FluxaAppleHeadlessResult.self, from: Data(json.utf8))
    }

    private func encode(_ completion: FluxaAppleHeadlessCompletion) throws -> String {
        String(decoding: try encoder.encode(completion), as: UTF8.self)
    }
}

actor FluxaAppleAsyncMutex {
    private var isLocked = false
    private var waiters = [CheckedContinuation<Void, Never>]()

    func acquire() async {
        if !isLocked {
            isLocked = true
            return
        }
        await withCheckedContinuation { continuation in
            waiters.append(continuation)
        }
    }

    func release() {
        if waiters.isEmpty {
            isLocked = false
        } else {
            waiters.removeFirst().resume()
        }
    }
}
