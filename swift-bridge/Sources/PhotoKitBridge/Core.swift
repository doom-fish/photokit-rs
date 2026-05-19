import AppKit
import CoreGraphics
import Foundation
import Photos

public let PKR_OK: Int32 = 0
public let PKR_ERROR: Int32 = -1

@_cdecl("ph_string_free")
public func ph_string_free(_ string: UnsafeMutablePointer<CChar>?) {
    guard let string else { return }
    free(string)
}

@inline(__always)
public func pkrCString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

@inline(__always)
public func pkrRetain<T: AnyObject>(_ object: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
public func pkrBorrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type = T.self) -> T {
    let typed = ptr.assumingMemoryBound(to: T.self)
    return Unmanaged<T>.fromOpaque(UnsafeRawPointer(typed)).takeUnretainedValue()
}

@inline(__always)
public func pkrRelease(_ ptr: UnsafeMutableRawPointer) {
    let typed = ptr.assumingMemoryBound(to: UInt8.self)
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(typed)).release()
}

public struct PKRErrorPayload: Codable {
    public var domain: String
    public var code: Int
    public var message: String
    public var localIdentifiers: [String]
}

public struct PKRRectPayload: Codable {
    public var x: Double
    public var y: Double
    public var width: Double
    public var height: Double
}

public struct PKRColorPayload: Codable {
    public var red: Double
    public var green: Double
    public var blue: Double
    public var alpha: Double
}

public func pkrRectPayload(_ rect: CGRect) -> PKRRectPayload {
    PKRRectPayload(
        x: Double(rect.origin.x),
        y: Double(rect.origin.y),
        width: Double(rect.size.width),
        height: Double(rect.size.height)
    )
}

public func pkrRect(from payload: PKRRectPayload) -> CGRect {
    CGRect(
        x: payload.x,
        y: payload.y,
        width: payload.width,
        height: payload.height
    )
}

public func pkrColorPayload(_ color: NSColor?) -> PKRColorPayload? {
    guard let color, let rgbColor = color.usingColorSpace(.deviceRGB) else { return nil }
    return PKRColorPayload(
        red: Double(rgbColor.redComponent),
        green: Double(rgbColor.greenComponent),
        blue: Double(rgbColor.blueComponent),
        alpha: Double(rgbColor.alphaComponent)
    )
}

private let pkrFractionalDateFormatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

public func pkrDateString(_ date: Date?) -> String? {
    guard let date else { return nil }
    return pkrFractionalDateFormatter.string(from: date)
}

public func pkrEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let encoder = JSONEncoder()
    let data = try encoder.encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "failed to encode JSON as UTF-8"]
        )
    }
    return string
}

public func pkrDecodeJSON<T: Decodable>(_ json: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let json else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "missing JSON payload"]
        )
    }

    let data = Data(String(cString: json).utf8)
    return try JSONDecoder().decode(T.self, from: data)
}

public func pkrErrorPayload(from error: Error) -> PKRErrorPayload {
    let nsError = error as NSError
    let localIdentifiers: [String] = {
        if #available(macOS 12.0, *), let identifiers = nsError.userInfo[PHLocalIdentifiersErrorKey] as? [String] {
            return identifiers
        }
        if #available(macOS 12.0, *), let identifiers = nsError.userInfo[PHLocalIdentifiersErrorKey] as? NSSet {
            return identifiers.compactMap { $0 as? String }
        }
        return []
    }()
    return PKRErrorPayload(
        domain: nsError.domain,
        code: nsError.code,
        message: nsError.localizedDescription,
        localIdentifiers: localIdentifiers
    )
}

public func pkrSetError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ error: Error
) {
    guard let outError else { return }

    if let json = try? pkrEncodeJSON(pkrErrorPayload(from: error)) {
        outError.pointee = pkrCString(json)
    } else {
        outError.pointee = pkrCString((error as NSError).localizedDescription)
    }
}

public func pkrSetMessageError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    message: String,
    domain: String = "photokit-rs",
    code: Int = -1
) {
    guard let outError else { return }
    let payload = PKRErrorPayload(domain: domain, code: code, message: message, localIdentifiers: [])
    if let json = try? pkrEncodeJSON(payload) {
        outError.pointee = pkrCString(json)
    } else {
        outError.pointee = pkrCString(message)
    }
}
