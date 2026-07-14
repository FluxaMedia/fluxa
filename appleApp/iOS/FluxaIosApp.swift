import FluxaShared
import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion: String
    private let headlessRuntime: FluxaAppleHeadlessRuntime
    private let appRuntime: FluxaAppleAppRuntime
    private let catalogStartup: FluxaAppleCatalogStartup

    init() {
        let runtime = requireFluxaAppleHeadlessRuntime()
        coreVersion = FluxaRustCoreRuntime.version()
        headlessRuntime = runtime
        appRuntime = FluxaAppleAppRuntime(runtime: runtime)
        catalogStartup = FluxaAppleCatalogStartup(coordinator: appRuntime.coordinator)
        let detailStartup = FluxaAppleDetailStartup(coordinator: appRuntime.coordinator)
        let searchStartup = FluxaAppleSearchStartup(coordinator: appRuntime.coordinator)
        let discoverStartup = FluxaAppleDiscoverStartup(coordinator: appRuntime.coordinator)
        let calendarStartup = FluxaAppleCalendarStartup(coordinator: appRuntime.coordinator)
        let libraryStartup = FluxaAppleLibraryStartup(coordinator: appRuntime.coordinator)
        let authStartup = FluxaAppleAuthStartup()
        FluxaApple.shared.setCatalogHomeRefreshHandler {
            Task { @MainActor in
                await catalogStartup.refresh()
            }
        }
        FluxaApple.shared.setSearchHandler { query in
            Task { @MainActor in
                await searchStartup.search(query: query)
            }
        }
        FluxaApple.shared.setDiscoverHandler { request in
            Task { @MainActor in
                await discoverStartup.discover(request: request)
            }
        }
        FluxaApple.shared.setCalendarMonthHandler { year, month in
            Task { @MainActor in
                await calendarStartup.load(year: year.intValue, month: month.intValue)
            }
        }
        FluxaApple.shared.setAuthSubmitHandler { request in
            Task { @MainActor in
                await authStartup.submit(request: request)
            }
        }
        FluxaApple.shared.setLibraryRefreshHandler {
            Task { @MainActor in
                await libraryStartup.refresh()
            }
        }
        FluxaApple.shared.setDetailHandlers(
            load: { request in
                Task { @MainActor in
                    await detailStartup.load(request: request)
                }
            },
            watchlist: { request in
                Task { @MainActor in
                    await detailStartup.toggleWatchlist(request: request)
                }
            }
        )
        FluxaApple.shared.setPlaybackHandler { request in
            Task { @MainActor in
                FluxaApplePlaybackPresenter.shared.present(request: request)
            }
        }
    }

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
        }
    }
}
