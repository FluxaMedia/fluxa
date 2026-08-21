import FluxaCore
import FluxaPlayer
import SwiftUI

@main
struct FluxaTvosApp: App {
    @StateObject private var homeModel: FluxaTvosHomeModel
    @AppStorage("fluxa.theme.pack") private var storedThemeJSON = ""

    init() {
        _homeModel = StateObject(
            wrappedValue: FluxaTvosHomeModel(runtime: requireFluxaAppleHeadlessRuntime())
        )
    }

    var body: some Scene {
        let theme = FluxaThemePacks.decode(storedThemeJSON.data(using: .utf8) ?? Data())
        let compactLayout = theme.layouts.home == "compact"
        WindowGroup {
            NavigationStack {
                ScrollView {
                    VStack(alignment: .leading, spacing: compactLayout ? 24 : 36) {
                        Text(FluxaTvos.shared.homeTitle())
                            .font(.largeTitle.bold())
                            .foregroundStyle(Color(themeHex: theme.colors.textPrimary, fallback: .white))
                        if homeModel.isLoading && homeModel.rows.isEmpty {
                            ProgressView()
                                .frame(maxWidth: .infinity, minHeight: 360)
                        }
                        ForEach(homeModel.rows) { row in
                            VStack(alignment: .leading, spacing: compactLayout ? 10 : 14) {
                                Text(row.title)
                                    .font(.title2.bold())
                                    .foregroundStyle(Color(themeHex: theme.colors.textPrimary, fallback: .white))
                                ScrollView(.horizontal) {
                                    LazyHStack(spacing: compactLayout ? 12 : 18) {
                                        ForEach(row.items) { item in
                                            VStack(alignment: .leading, spacing: 8) {
                                                AsyncImage(url: item.artworkUrl) { image in
                                                    image.resizable().scaledToFill()
                                                } placeholder: {
                                                    Color(themeHex: theme.colors.surfaceRaised, fallback: Color.gray.opacity(0.25))
                                                }
                                                .frame(width: 190, height: 285)
                                                .clipShape(RoundedRectangle(cornerRadius: theme.shape.cardRadius))
                                                Text(item.title)
                                                    .font(.headline)
                                                    .foregroundStyle(Color(themeHex: theme.colors.textPrimary, fallback: .white))
                                                    .lineLimit(1)
                                                if !item.subtitle.isEmpty {
                                                    Text(item.subtitle)
                                                        .font(.subheadline)
                                                        .foregroundStyle(Color(themeHex: theme.colors.textSecondary, fallback: .secondary))
                                                        .lineLimit(1)
                                                }
                                            }
                                            .frame(width: 190, alignment: .leading)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    .padding(48)
                }
            }
            .environment(\.fluxaTheme, theme)
            .background(Color(themeHex: theme.colors.background, fallback: .black))
            .task {
                await homeModel.load()
            }
        }
    }
}
