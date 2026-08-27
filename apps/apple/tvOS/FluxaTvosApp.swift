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
                                            NavigationLink {
                                                FluxaTvosDetailView(item: item, model: homeModel)
                                            } label: {
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
                                            .buttonStyle(.plain)
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

private struct FluxaTvosDetailView: View {
    let item: FluxaTvosHomeModel.Item
    @ObservedObject var model: FluxaTvosHomeModel
    @State private var detail: FluxaTvosHomeModel.Detail?
    @State private var loading = true

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                Text(detail?.title ?? item.title)
                    .font(.largeTitle.bold())
                if let description = detail?.description, !description.isEmpty {
                    Text(description)
                        .font(.body)
                }
                if loading {
                    ProgressView()
                } else if let detail, !detail.episodes.isEmpty {
                    ForEach(detail.episodes) { episode in
                        Button {
                            Task { @MainActor in
                                let options = await model.playbackOptions(for: item, contentId: episode.id)
                                FluxaTvosPlaybackPresenter.shared.present(options: options, title: episode.title)
                            }
                        } label: {
                            HStack {
                                Text(episode.title)
                                if !episode.subtitle.isEmpty {
                                    Text(episode.subtitle)
                                        .foregroundStyle(.secondary)
                                }
                            }
                        }
                    }
                } else {
                    Button(item.title) {
                        Task { @MainActor in
                            let options = await model.playbackOptions(for: item)
                            FluxaTvosPlaybackPresenter.shared.present(options: options, title: item.title)
                        }
                    }
                }
            }
            .padding(48)
        }
        .task {
            detail = await model.detail(for: item)
            loading = false
        }
    }
}
