import Foundation
#if canImport(Darwin)
import Darwin
#endif

enum FluxaAppleNetGuard {
    static func resolveAllowedAddresses(host: String) -> [[UInt8]]? {
        var hints = addrinfo(
            ai_flags: 0,
            ai_family: AF_UNSPEC,
            ai_socktype: SOCK_STREAM,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_canonname: nil,
            ai_addr: nil,
            ai_next: nil
        )
        var resultPointer: UnsafeMutablePointer<addrinfo>?
        let status = getaddrinfo(host, nil, &hints, &resultPointer)
        guard status == 0, let firstResult = resultPointer else { return nil }
        defer { freeaddrinfo(resultPointer) }

        var addresses: [[UInt8]] = []
        var pointer: UnsafeMutablePointer<addrinfo>? = firstResult
        while let info = pointer {
            if let addr = info.pointee.ai_addr {
                if info.pointee.ai_family == AF_INET {
                    let sockaddrIn = addr.withMemoryRebound(to: sockaddr_in.self, capacity: 1) { $0.pointee }
                    var inAddr = sockaddrIn.sin_addr
                    let bytes = withUnsafeBytes(of: &inAddr) { Array($0) }
                    addresses.append(bytes)
                } else if info.pointee.ai_family == AF_INET6 {
                    let sockaddrIn6 = addr.withMemoryRebound(to: sockaddr_in6.self, capacity: 1) { $0.pointee }
                    var in6Addr = sockaddrIn6.sin6_addr
                    let bytes = withUnsafeBytes(of: &in6Addr) { Array($0) }
                    addresses.append(bytes)
                }
            }
            pointer = info.pointee.ai_next
        }

        guard !addresses.isEmpty, !addresses.contains(where: isBlockedAddress) else { return nil }
        return addresses
    }

    static func isBlockedAddress(_ bytes: [UInt8]) -> Bool {
        !FluxaCoreStremio.pluginNetworkAddressBytesAllowed(bytes)
    }
}
