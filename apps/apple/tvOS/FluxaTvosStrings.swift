import Foundation

enum FluxaTvosStrings {
    private static let table: [String: String] = {
        let file = Locale.current.language.languageCode?.identifier == "tr" ? "tr_tr" : "english_us"
        guard let url = Bundle.main.url(forResource: file, withExtension: "json")
                ?? Bundle.main.url(forResource: file, withExtension: "json", subdirectory: "i18n"),
              let data = try? Data(contentsOf: url),
              let table = try? JSONDecoder().decode([String: String].self, from: data) else {
            return [:]
        }
        return table
    }()

    static func text(_ key: String) -> String {
        table[key] ?? key
    }
}
