import FluxaCore
import FluxaPlayer
import SwiftUI

@main
struct FluxaTvosApp: App {
    @StateObject private var homeModel: FluxaTvosHomeModel

    init() {
        _homeModel = StateObject(
            wrappedValue: FluxaTvosHomeModel(runtime: requireFluxaAppleHeadlessRuntime())
        )
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                ScrollView {
                    VStack(alignment: .leading, spacing: 36) {
                        Text(FluxaTvos.shared.homeTitle())
                            .font(.largeTitle.bold())
                        if homeModel.isLoading && homeModel.rows.isEmpty {
                            ProgressView()
                                .frame(maxWidth: .infinity, minHeight: 360)
                        }
                        ForEach(homeModel.rows) { row in
                            VStack(alignment: .leading, spacing: 14) {
                                Text(row.title)
                                    .font(.title2.bold())
                                ScrollView(.horizontal) {
                                    LazyHStack(spacing: 18) {
                                        ForEach(row.items) { item in
                                            VStack(alignment: .leading, spacing: 8) {
                                                AsyncImage(url: item.artworkUrl) { image in
                                                    image.resizable().scaledToFill()
                                                } placeholder: {
                                                    Color.gray.opacity(0.25)
                                                }
                                                .frame(width: 190, height: 285)
                                                .clipShape(RoundedRectangle(cornerRadius: 12))
                                                Text(item.title)
                                                    .font(.headline)
                                                    .lineLimit(1)
                                                if !item.subtitle.isEmpty {
                                                    Text(item.subtitle)
                                                        .font(.subheadline)
                                                        .foregroundStyle(.secondary)
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
            .task {
                await homeModel.load()
            }
        }
    }
}
