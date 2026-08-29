import FluxaPlayerKit

@MainActor
final class FluxaAppleDiscordPresence {
    private let applicationId: UInt64 = 1518004842860122174

    func start() {
        FluxaDiscordPresenceInitialize(applicationId)
        FluxaDiscordPresenceAuthorize()
    }

    func publish(_ snapshot: FluxaDiscordPresenceSnapshot) {
        withCStringOrNil(snapshot.artworkURL?.absoluteString) { artwork in
            snapshot.title.withCString { title in
                snapshot.episodeLine.withCString { episodeLine in
                    snapshot.status.withCString { status in
                        FluxaDiscordPresenceUpdate(
                            title,
                            episodeLine,
                            status,
                            Int64(snapshot.position * 1000),
                            Int64(snapshot.duration * 1000),
                            artwork
                        )
                    }
                }
            }
        }
    }

    func clear() {
        FluxaDiscordPresenceClear()
    }

    private func withCStringOrNil<T>(_ value: String?, _ body: (UnsafePointer<CChar>?) -> T) -> T {
        guard let value else { return body(nil) }
        return value.withCString(body)
    }
}
