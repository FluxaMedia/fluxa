import FluxaShared
import Foundation

@MainActor
final class FluxaAppleAddonStoreStartup {
    private let manager: FluxaAppleAddonStoreManager

    init(manager: FluxaAppleAddonStoreManager) {
        self.manager = manager
    }

    func start() async {
        push(addons: await manager.currentAddons())
    }

    func handle(_ action: AppleAddonStoreActionSnapshot) async {
        switch action.type {
        case "refresh":
            push(addons: await manager.currentAddons())
        case "submitInput":
            guard let text = action.text, !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
                return
            }
            let result = await manager.submitManifestUrl(text)
            push(
                addons: result.addons,
                addedAddonName: result.addedName,
                inputFailed: result.failed,
                clearInputOnSuccess: !result.failed
            )
        case "toggleAddon":
            guard let url = action.url, let enabled = action.enabled?.boolValue else { return }
            push(addons: await manager.toggleAddon(url: url, enabled: enabled))
        case "removeAddon":
            guard let url = action.url else { return }
            push(addons: await manager.removeAddon(url: url))
        case "moveAddon":
            guard let url = action.url, let direction = action.direction?.intValue else { return }
            push(addons: await manager.moveAddon(url: url, direction: direction))
        case "refreshAddon":
            guard let url = action.url else { return }
            push(addons: await manager.currentAddons(), refreshingUrl: url)
            push(addons: await manager.refreshAddon(url: url))
        default:
            break
        }
    }

    private func push(
        addons: [FluxaAppleInstalledAddon],
        refreshingUrl: String? = nil,
        isLoading: Bool = false,
        addedAddonName: String? = nil,
        inputFailed: Bool = false,
        clearInputOnSuccess: Bool = false
    ) {
        let snapshot = AppleAddonStoreSnapshot(
            installedAddons: addons.map { addon in
                AppleInstalledAddonSnapshot(
                    name: addon.name,
                    description: addon.description,
                    url: addon.url,
                    logoUrl: addon.logoUrl,
                    version: addon.version,
                    configurable: addon.configurable,
                    isEnabled: addon.isEnabled,
                    canMoveUp: addon.canMoveUp,
                    canMoveDown: addon.canMoveDown,
                    isRefreshing: addon.url == refreshingUrl
                )
            },
            isLoading: isLoading,
            isSubmittingInput: false,
            inputFailed: inputFailed,
            addedAddonName: addedAddonName,
            clearInputOnSuccess: clearInputOnSuccess
        )
        FluxaApple.shared.updateAddonStore(snapshot: snapshot)
    }
}
