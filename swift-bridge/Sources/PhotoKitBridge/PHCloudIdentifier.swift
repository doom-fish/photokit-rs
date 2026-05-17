import Foundation
import Photos

struct PKRCloudIdentifierPayload: Codable, Hashable {
    var stringValue: String
}

struct PKRCloudIdentifierMappingPayload: Codable {
    var cloudIdentifier: PKRCloudIdentifierPayload?
    var error: PKRErrorPayload?
}

struct PKRLocalIdentifierMappingPayload: Codable {
    var localIdentifier: String?
    var error: PKRErrorPayload?
}

struct PKRCloudIdentifierMappingEntryPayload: Codable {
    var localIdentifier: String
    var mapping: PKRCloudIdentifierMappingPayload
}

struct PKRLocalIdentifierMappingEntryPayload: Codable {
    var cloudIdentifier: PKRCloudIdentifierPayload
    var mapping: PKRLocalIdentifierMappingPayload
}

func pkrEncodeCloudIdentifier(_ identifier: PHCloudIdentifier?) -> PKRCloudIdentifierPayload? {
    guard let identifier else { return nil }
    return PKRCloudIdentifierPayload(stringValue: identifier.stringValue)
}

func pkrNotFoundErrorPayload(_ message: String) -> PKRErrorPayload {
    PKRErrorPayload(domain: "photokit-rs", code: -1, message: message, localIdentifiers: [])
}

@_cdecl("ph_photo_library_cloud_identifier_mappings_json")
public func ph_photo_library_cloud_identifier_mappings_json(
    _ library: UnsafeMutableRawPointer?,
    _ localIdentifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    do {
        let localIdentifiers = try pkrDecodeJSON(localIdentifiersJSON, as: [String].self)
        let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
        let cloudIdentifiers = photoLibrary.cloudIdentifiers(forLocalIdentifiers: localIdentifiers)
        let payload = zip(localIdentifiers, cloudIdentifiers).map { localIdentifier, cloudIdentifier in
            PKRCloudIdentifierMappingEntryPayload(
                localIdentifier: localIdentifier,
                mapping: PKRCloudIdentifierMappingPayload(
                    cloudIdentifier: pkrEncodeCloudIdentifier(cloudIdentifier),
                    error: nil
                )
            )
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_photo_library_local_identifier_mappings_json")
public func ph_photo_library_local_identifier_mappings_json(
    _ library: UnsafeMutableRawPointer?,
    _ cloudIdentifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    do {
        let payloads = try pkrDecodeJSON(cloudIdentifiersJSON, as: [PKRCloudIdentifierPayload].self)
        let cloudIdentifiers = payloads.map { PHCloudIdentifier(stringValue: $0.stringValue) }
        let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
        let localIdentifiers = photoLibrary.localIdentifiers(for: cloudIdentifiers)
        let payload = zip(payloads, localIdentifiers).map { originalPayload, localIdentifier in
            PKRLocalIdentifierMappingEntryPayload(
                cloudIdentifier: originalPayload,
                mapping: PKRLocalIdentifierMappingPayload(localIdentifier: localIdentifier, error: nil)
            )
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
