import Foundation

private final class FluxaApplePluginRedirectBlockingDelegate: NSObject, URLSessionTaskDelegate {
    func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        willPerformHTTPRedirection response: HTTPURLResponse,
        newRequest request: URLRequest,
        completionHandler: @escaping (URLRequest?) -> Void
    ) {
        completionHandler(nil)
    }
}

final class FluxaApplePluginHttpClient: PluginHttpClient {
    private let session: URLSession
    private let delegate = FluxaApplePluginRedirectBlockingDelegate()
    private let maxRedirects = 10

    init() {
        session = URLSession(configuration: .ephemeral, delegate: delegate, delegateQueue: nil)
    }

    func fetch(request: PluginHttpRequest) -> PluginHttpResponse {
        var currentUrl = request.url
        var redirectsLeft = request.followRedirects ? maxRedirects : 0

        while true {
            guard let url = URL(string: currentUrl), FluxaAppleNetGuard.isSchemeAllowed(url.scheme) else {
                return blockedResponse("unsupported scheme or invalid url")
            }
            guard let host = url.host, FluxaAppleNetGuard.resolveAllowedAddresses(host: host) != nil else {
                return blockedResponse("blocked address for host \(url.host ?? currentUrl)")
            }

            let method = request.method.isEmpty ? "GET" : request.method.uppercased()
            var urlRequest = URLRequest(url: url)
            urlRequest.httpMethod = method
            for (key, value) in request.headers {
                urlRequest.setValue(value, forHTTPHeaderField: key)
            }
            if let body = request.body, method != "GET", method != "HEAD" {
                urlRequest.httpBody = Data(body.utf8)
            }

            let (data, response, error) = performSynchronously(urlRequest)
            if let error {
                return blockedResponse(error.localizedDescription)
            }
            guard let httpResponse = response as? HTTPURLResponse else {
                return blockedResponse("no response")
            }

            if (300..<400).contains(httpResponse.statusCode), redirectsLeft > 0,
               let location = httpResponse.value(forHTTPHeaderField: "Location"),
               let redirectUrl = URL(string: location, relativeTo: url) {
                currentUrl = redirectUrl.absoluteString
                redirectsLeft -= 1
                continue
            }

            let body = data.map { String(decoding: $0, as: UTF8.self) } ?? ""
            var headers: [String: String] = [:]
            for (key, value) in httpResponse.allHeaderFields {
                if let keyString = key as? String, let valueString = value as? String {
                    headers[keyString] = valueString
                }
            }
            let ok = (200..<300).contains(httpResponse.statusCode)
            return PluginHttpResponse(
                status: UInt16(clamping: httpResponse.statusCode),
                headers: headers,
                body: body,
                ok: ok,
                error: ok ? nil : "http_\(httpResponse.statusCode)"
            )
        }
    }

    private func performSynchronously(_ request: URLRequest) -> (Data?, URLResponse?, Error?) {
        let semaphore = DispatchSemaphore(value: 0)
        var resultData: Data?
        var resultResponse: URLResponse?
        var resultError: Error?
        session.dataTask(with: request) { data, response, error in
            resultData = data
            resultResponse = response
            resultError = error
            semaphore.signal()
        }.resume()
        semaphore.wait()
        return (resultData, resultResponse, resultError)
    }

    private func blockedResponse(_ reason: String) -> PluginHttpResponse {
        PluginHttpResponse(status: 0, headers: [:], body: "", ok: false, error: reason)
    }
}
