import Foundation

public struct FluxaSubtitleCue: Sendable, Equatable {
    public let start: TimeInterval
    public let end: TimeInterval
    public let text: String

    public init(start: TimeInterval, end: TimeInterval, text: String) {
        self.start = start
        self.end = end
        self.text = text
    }
}

enum FluxaSubtitleParser {
    static func parse(_ data: Data) -> [FluxaSubtitleCue] {
        guard let source = String(data: data, encoding: .utf8) else { return [] }
        let lines = source.replacingOccurrences(of: "\r\n", with: "\n")
            .replacingOccurrences(of: "\r", with: "\n")
            .components(separatedBy: "\n")
        var cues: [FluxaSubtitleCue] = []
        var index = 0

        while index < lines.count {
            let line = lines[index].trimmingCharacters(in: .whitespaces)
            if line.isEmpty || line.uppercased() == "WEBVTT" {
                index += 1
                continue
            }
            let uppercasedLine = line.uppercased()
            if uppercasedLine == "NOTE" || uppercasedLine.hasPrefix("NOTE ") {
                index += 1
                while index < lines.count,
                      !lines[index].trimmingCharacters(in: .whitespaces).isEmpty {
                    index += 1
                }
                continue
            }

            let timingLine: String
            if line.contains("-->") {
                timingLine = line
            } else if index + 1 < lines.count, lines[index + 1].contains("-->") {
                index += 1
                timingLine = lines[index]
            } else {
                index += 1
                continue
            }

            let parts = timingLine.components(separatedBy: "-->")
            guard parts.count >= 2,
                  let start = parseTime(parts[0]),
                  let end = parseTime(parts[1].split(separator: " ", maxSplits: 1).first.map(String.init) ?? "") else {
                index += 1
                continue
            }
            index += 1
            var textLines: [String] = []
            while index < lines.count, !lines[index].trimmingCharacters(in: .whitespaces).isEmpty {
                textLines.append(lines[index])
                index += 1
            }
            let text = textLines.joined(separator: "\n")
                .replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression)
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if !text.isEmpty, end > start {
                cues.append(FluxaSubtitleCue(start: start, end: end, text: text))
            }
        }
        return cues.sorted { $0.start < $1.start }
    }

    private static func parseTime(_ value: String) -> TimeInterval? {
        let normalized = value.trimmingCharacters(in: .whitespaces)
            .replacingOccurrences(of: ",", with: ".")
        let parts = normalized.split(separator: ":").map(String.init)
        guard parts.count == 2 || parts.count == 3 else { return nil }
        let seconds: Double
        if parts.count == 2 {
            guard let minutes = Double(parts[0]), let rest = Double(parts[1]) else { return nil }
            seconds = minutes * 60 + rest
        } else {
            guard let hours = Double(parts[0]), let minutes = Double(parts[1]), let rest = Double(parts[2]) else {
                return nil
            }
            seconds = hours * 3600 + minutes * 60 + rest
        }
        return seconds.isFinite && seconds >= 0 ? seconds : nil
    }
}
