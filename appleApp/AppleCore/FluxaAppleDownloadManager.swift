import Foundation
import FluxaShared

struct FluxaAppleDownloadItem: Codable {
    let id: String
    let metaId: String
    let metaType: String
    var title: String
    var episodeTitle: String?
    var videoId: String?
    var posterUrl: String?
    var backgroundUrl: String?
    var videoPath: String = ""
    var status: String = "queued"
    var progress: Int = 0
    var downloadedBytes: Int64 = 0
    var totalBytes: Int64 = 0
    var errorMessage: String?
}

/// Background-session download engine backing the iOS "downloads" library section.
/// Note: this wires a background `URLSession` for in-flight downloads to survive the app
/// being backgrounded, but does NOT implement `application(_:handleEventsForBackgroundURLSession:completionHandler:)`
/// (the AppDelegate hook iOS uses to relaunch a fully-terminated app and report finished
/// background transfers). That's the standard remaining piece for downloads to reliably
/// complete/report after the app is killed by the OS - needs a real device to get right.
@MainActor
final class FluxaAppleDownloadManager: NSObject {
    static let shared = FluxaAppleDownloadManager()

    private let defaultsKey = "fluxa_offline_downloads"
    private let defaults: UserDefaults
    private var items: [String: FluxaAppleDownloadItem] = [:]
    private var tasksByItemId: [String: URLSessionDownloadTask] = [:]
    private var itemIdByTaskId: [Int: String] = [:]
    private var language: String = "en"

    private lazy var session: URLSession = {
        let config = URLSessionConfiguration.background(withIdentifier: "com.fluxa.app.downloads")
        config.isDiscretionary = false
        config.sessionSendsLaunchEvents = true
        return URLSession(configuration: config, delegate: self, delegateQueue: nil)
    }()

    private override init() {
        self.defaults = .standard
        super.init()
        load()
    }

    func setLanguage(_ language: String) {
        self.language = language
        pushState()
    }

    func downloadedFilePath(forVideoId videoId: String) -> String? {
        items[videoId]?.videoPath.isEmpty == false ? items[videoId]?.videoPath : nil
    }

    func enqueue(
        metaId: String,
        metaType: String,
        title: String,
        episodeTitle: String?,
        videoId: String?,
        posterUrl: String?,
        backgroundUrl: String?,
        streamUrl: String,
        requestHeaders: [String: String]
    ) {
        let id = videoId ?? metaId
        if let existing = items[id], existing.status == "downloading" || existing.status == "downloaded" {
            return
        }
        var item = FluxaAppleDownloadItem(
            id: id,
            metaId: metaId,
            metaType: metaType,
            title: title,
            episodeTitle: episodeTitle,
            videoId: videoId,
            posterUrl: posterUrl,
            backgroundUrl: backgroundUrl
        )

        guard let url = URL(string: streamUrl) else {
            item.status = "failed"
            item.errorMessage = "Invalid stream URL"
            items[id] = item
            save()
            pushState()
            return
        }

        item.status = "downloading"
        items[id] = item
        save()
        pushState()

        var request = URLRequest(url: url)
        for (key, value) in requestHeaders {
            request.setValue(value, forHTTPHeaderField: key)
        }
        let task = session.downloadTask(with: request)
        tasksByItemId[id] = task
        itemIdByTaskId[task.taskIdentifier] = id
        task.resume()
    }

    func cancel(id: String) {
        tasksByItemId[id]?.cancel()
        tasksByItemId[id] = nil
        if let item = items[id], !item.videoPath.isEmpty {
            try? FileManager.default.removeItem(atPath: item.videoPath)
        }
        items[id] = nil
        save()
        pushState()
    }

    private func load() {
        guard let data = defaults.data(forKey: defaultsKey),
              let decoded = try? JSONDecoder().decode([String: FluxaAppleDownloadItem].self, from: data) else {
            return
        }
        // Any item still "downloading" across a process relaunch has no live URLSessionTask
        // backing it anymore (background-session task reattachment isn't implemented), so
        // surface it as failed rather than showing a permanently stuck progress bar.
        items = decoded.mapValues { item in
            var item = item
            if item.status == "downloading" {
                item.status = "failed"
                item.errorMessage = "Download interrupted"
            }
            return item
        }
    }

    private func save() {
        guard let data = try? JSONEncoder().encode(items) else { return }
        defaults.set(data, forKey: defaultsKey)
    }

    private func pushState() {
        let snapshots = items.values.map { item in
            FluxaShared.AppleOfflineDownloadItemSnapshot(
                id: item.id,
                metaId: item.metaId,
                metaType: item.metaType,
                title: item.title,
                episodeTitle: item.episodeTitle,
                videoId: item.videoId,
                posterUrl: item.posterUrl,
                backgroundUrl: item.backgroundUrl,
                videoPath: item.videoPath,
                status: item.status,
                progress: Int32(item.progress),
                downloadedBytes: item.downloadedBytes,
                totalBytes: item.totalBytes,
                errorMessage: item.errorMessage
            )
        }
        FluxaApple.shared.updateDownloads(items: snapshots, language: language)
    }
}

extension FluxaAppleDownloadManager: URLSessionDownloadDelegate {
    nonisolated func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didWriteData bytesWritten: Int64,
        totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        let taskId = downloadTask.taskIdentifier
        Task { @MainActor in
            guard let id = itemIdByTaskId[taskId], var item = items[id] else { return }
            item.downloadedBytes = totalBytesWritten
            if totalBytesExpectedToWrite > 0 {
                item.totalBytes = totalBytesExpectedToWrite
            }
            item.progress = item.totalBytes > 0 ? Int((totalBytesWritten * 100) / item.totalBytes) : 0
            items[id] = item
            pushState()
        }
    }

    nonisolated func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didFinishDownloadingTo location: URL
    ) {
        let taskId = downloadTask.taskIdentifier
        // Must move the temp file synchronously here - iOS deletes it right after this
        // delegate callback returns, so this cannot be deferred onto the main actor.
        let directory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("offline", isDirectory: true)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let target = directory.appendingPathComponent("\(taskId).mp4")
        try? FileManager.default.removeItem(at: target)
        let moveError: Error?
        do {
            try FileManager.default.moveItem(at: location, to: target)
            moveError = nil
        } catch {
            moveError = error
        }
        Task { @MainActor in
            guard let id = itemIdByTaskId[taskId], var item = items[id] else { return }
            if let moveError {
                item.status = "failed"
                item.errorMessage = moveError.localizedDescription
            } else {
                item.videoPath = target.path
                item.status = "downloaded"
                item.progress = 100
            }
            items[id] = item
            tasksByItemId[id] = nil
            itemIdByTaskId[taskId] = nil
            save()
            pushState()
        }
    }

    nonisolated func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        didCompleteWithError error: Error?
    ) {
        guard let error else { return }
        let taskId = task.taskIdentifier
        Task { @MainActor in
            guard let id = itemIdByTaskId[taskId], var item = items[id] else { return }
            if item.status != "downloaded" {
                item.status = "failed"
                item.errorMessage = error.localizedDescription
                items[id] = item
                save()
                pushState()
            }
            tasksByItemId[id] = nil
            itemIdByTaskId[taskId] = nil
        }
    }
}
