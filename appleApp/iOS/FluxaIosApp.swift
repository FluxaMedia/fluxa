import FluxaShared
import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion: String
    private let headlessRuntime: FluxaAppleHeadlessRuntime
    private let appRuntime: FluxaAppleAppRuntime
    private let catalogStartup: FluxaAppleCatalogStartup
    private let pluginsManager: FluxaApplePluginRepositoryManager
    private let pluginsStartup: FluxaApplePluginsStartup
    private let addonStoreManager: FluxaAppleAddonStoreManager
    private let addonStoreStartup: FluxaAppleAddonStoreStartup

    init() {
        let runtime = requireFluxaAppleHeadlessRuntime()
        coreVersion = FluxaRustCoreRuntime.version()
        headlessRuntime = runtime
        let runtimeApp = FluxaAppleAppRuntime(runtime: runtime)
        appRuntime = runtimeApp
        let catalogStartup = FluxaAppleCatalogStartup(coordinator: runtimeApp.coordinator)
        self.catalogStartup = catalogStartup
        let detailStartup = FluxaAppleDetailStartup(coordinator: runtimeApp.coordinator)
        let searchStartup = FluxaAppleSearchStartup(coordinator: runtimeApp.coordinator)
        let discoverStartup = FluxaAppleDiscoverStartup(coordinator: runtimeApp.coordinator)
        let calendarStartup = FluxaAppleCalendarStartup(coordinator: runtimeApp.coordinator)
        let libraryStartup = FluxaAppleLibraryStartup(coordinator: runtimeApp.coordinator)
        let authStartup = FluxaAppleAuthStartup()
        let pluginsManager = FluxaApplePluginRepositoryManager()
        self.pluginsManager = pluginsManager
        let pluginsStartup = FluxaApplePluginsStartup(manager: pluginsManager)
        self.pluginsStartup = pluginsStartup
        let addonStoreManager = FluxaAppleAddonStoreManager()
        self.addonStoreManager = addonStoreManager
        let addonStoreStartup = FluxaAppleAddonStoreStartup(manager: addonStoreManager)
        self.addonStoreStartup = addonStoreStartup
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
        FluxaApple.shared.setPluginsActionHandler { action in
            Task { @MainActor in
                await pluginsStartup.handle(action)
            }
        }
        Task { @MainActor in
            await pluginsStartup.start()
        }
        FluxaApple.shared.setAddonStoreActionHandler { action in
            Task { @MainActor in
                await addonStoreStartup.handle(action)
            }
        }
        Task { @MainActor in
            await addonStoreStartup.start()
        }
    }

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
        }
    }
}
