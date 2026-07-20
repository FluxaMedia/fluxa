import Foundation
#if canImport(Darwin)
import Darwin
#endif

enum FluxaAppleNetGuard {
    static func isSchemeAllowed(_ scheme: String?) -> Bool {
        guard let scheme = scheme?.lowercased() else { return false }
        return scheme == "http" || scheme == "https"
    }

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
        switch bytes.count {
        case 4:
            return isBlockedIPv4(bytes)
        case 16:
            if let mapped = ipv4Mapped(bytes) {
                return isBlockedIPv4(mapped)
            }
            return isBlockedIPv6(bytes)
        default:
            return true
        }
    }

    private static func isBlockedIPv4(_ bytes: [UInt8]) -> Bool {
        guard bytes.count == 4 else { return true }
        let a = Int(bytes[0])
        let b = Int(bytes[1])
        if a == 127 { return true }
        if a == 10 { return true }
        if a == 172, (16...31).contains(b) { return true }
        if a == 192, b == 168 { return true }
        if a == 169, b == 254 { return true }
        if bytes == [0, 0, 0, 0] { return true }
        if bytes == [255, 255, 255, 255] { return true }
        if a == 100, (64...127).contains(b) { return true }
        return false
    }

    private static func isBlockedIPv6(_ bytes: [UInt8]) -> Bool {
        guard bytes.count == 16 else { return true }
        if bytes.allSatisfy({ $0 == 0 }) { return true }
        if bytes[0..<15].allSatisfy({ $0 == 0 }), bytes[15] == 1 { return true }
        let firstSegment = (Int(bytes[0]) << 8) | Int(bytes[1])
        if (firstSegment & 0xfe00) == 0xfc00 { return true }
        if (firstSegment & 0xffc0) == 0xfe80 { return true }
        return false
    }

    private static func ipv4Mapped(_ bytes: [UInt8]) -> [UInt8]? {
        guard bytes.count == 16 else { return nil }
        guard bytes[0..<10].allSatisfy({ $0 == 0 }), bytes[10] == 0xFF, bytes[11] == 0xFF else { return nil }
        return Array(bytes[12..<16])
    }
}
