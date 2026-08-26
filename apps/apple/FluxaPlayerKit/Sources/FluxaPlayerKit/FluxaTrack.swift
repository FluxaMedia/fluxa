import Foundation

public enum FluxaTrackKind: String, Sendable {
    case audio
    case subtitle
}

public struct FluxaTrack: Sendable, Identifiable, Equatable {
    public let id: String
    public let kind: FluxaTrackKind
    public let label: String
    public let languageCode: String?
    public let isForced: Bool

    public init(id: String, kind: FluxaTrackKind, label: String, languageCode: String?, isForced: Bool = false) {
        self.id = id
        self.kind = kind
        self.label = label
        self.languageCode = languageCode
        self.isForced = isForced
    }
}
