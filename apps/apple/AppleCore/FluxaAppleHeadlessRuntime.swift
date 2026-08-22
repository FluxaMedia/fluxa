import Foundation

final class FluxaAppleHeadlessRuntime {
    private let handle: Int64
    private let lock = NSLock()

    init?(initialStateJson: String = "{}") {
        let createdHandle = createHeadlessEngineJson(initialJson: initialStateJson)
        guard createdHandle > 0 else {
            return nil
        }
        handle = createdHandle
    }

    deinit {
        _ = destroyHeadlessEngineJson(handle: handle)
    }

    func snapshot() -> String {
        lock.withLock {
            headlessEngineSnapshotJson(handle: handle)
        }
    }

    func dispatch(actionJson: String) -> String {
        lock.withLock {
            headlessEngineDispatchJson(handle: handle, actionJson: actionJson)
        }
    }

    func completeEffect(resultJson: String) -> String {
        lock.withLock {
            headlessEngineCompleteEffectJson(handle: handle, resultJson: resultJson)
        }
    }
}

func requireFluxaAppleHeadlessRuntime() -> FluxaAppleHeadlessRuntime {
    guard let runtime = FluxaAppleHeadlessRuntime() else {
        fatalError("Fluxa Rust core headless runtime is unavailable.")
    }
    return runtime
}
