import FluxaShared
import Foundation

@MainActor
final class FluxaAppleCalendarStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func load(year: Int, month: Int) async {
        do {
            let result = try await coordinator.dispatch(
                actionJson: "{\"type\":\"calendarMonthRequested\",\"profile\":{\"id\":\"apple-default\"},\"year\":\(year),\"month\":\(month)}"
            )
            let items: [FluxaAppleJsonValue]
            if case .object(let calendar)? = result.state["calendar"],
               case .array(let values)? = calendar["items"] {
                items = values
            } else {
                items = []
            }
            updateSharedCalendar(year: year, month: month, items: items.compactMap(sharedItem))
        } catch {
            updateSharedCalendar(year: year, month: month, items: [])
        }
    }

    private func updateSharedCalendar(year: Int, month: Int, items: [AppleCalendarReleaseSnapshot]) {
        FluxaApple.shared.updateCalendar(snapshot: AppleCalendarSnapshot(year: Int32(year), month: Int32(month), items: items, isLoading: false))
    }

    private func sharedItem(_ value: FluxaAppleJsonValue) -> AppleCalendarReleaseSnapshot? {
        guard case .object(let item) = value,
              case .object(let meta)? = item["meta"],
              let id = string(meta["id"]),
              let title = string(item["title"]) ?? string(meta["name"]) else {
            return nil
        }
        return AppleCalendarReleaseSnapshot(id: id, type: string(meta["type"]) ?? "", dateIso: string(item["dateIso"]) ?? "", title: title, subtitle: string(item["subtitle"]) ?? "", posterUrl: string(item["episodePoster"] ?? item["poster"] ?? meta["poster"]), addonTransportUrl: string(meta["addonTransportUrl"]), catalogType: string(meta["catalogType"]))
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }

}
