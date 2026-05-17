import Foundation
import Photos

final class PKRPhotoLibraryBox: NSObject {
    let library: PHPhotoLibrary

    init(library: PHPhotoLibrary) {
        self.library = library
        super.init()
    }
}

public typealias PKRChangeObserverCallback = @convention(c) (UnsafeMutableRawPointer?, UnsafeMutableRawPointer?) -> Void

public typealias PKRAvailabilityObserverCallback = @convention(c) (UnsafeMutablePointer<CChar>?, UnsafeMutableRawPointer?) -> Void

final class PKRAvailabilityObserverBox: NSObject, PHPhotoLibraryAvailabilityObserver {
    let library: PHPhotoLibrary
    let callback: PKRAvailabilityObserverCallback
    let userInfo: UnsafeMutableRawPointer?

    init(
        library: PHPhotoLibrary,
        callback: @escaping PKRAvailabilityObserverCallback,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.library = library
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    func photoLibraryDidBecomeUnavailable(_ photoLibrary: PHPhotoLibrary) {
        let payload = PHPhotoLibraryAvailabilityChangePayload(
            unavailabilityReason: photoLibrary.unavailabilityReason.map(pkrErrorPayload)
        )
        let json = try? pkrEncodeJSON(payload)
        callback(json.flatMap(pkrCString), userInfo)
    }
}

struct PHPhotoLibraryAvailabilityChangePayload: Codable {
    var unavailabilityReason: PKRErrorPayload?
}

struct PKRPersistentChangeTokenPayload: Codable {
    var dataBase64: String
}

struct PKRPersistentObjectChangeDetailsPayload: Codable {
    var objectType: Int
    var insertedLocalIdentifiers: [String]
    var updatedLocalIdentifiers: [String]
    var deletedLocalIdentifiers: [String]
}

struct PKRPersistentChangePayload: Codable {
    var changeToken: PKRPersistentChangeTokenPayload
    var changeDetails: [PKRPersistentObjectChangeDetailsPayload]
}

struct PKRPersistentChangeFetchResultPayload: Codable {
    var changes: [PKRPersistentChangePayload]
}

func pkrEncodePersistentChangeToken(_ token: PHPersistentChangeToken) throws -> PKRPersistentChangeTokenPayload {
    let data = try NSKeyedArchiver.archivedData(withRootObject: token, requiringSecureCoding: true)
    return PKRPersistentChangeTokenPayload(dataBase64: data.base64EncodedString())
}

func pkrDecodePersistentChangeToken(_ payload: PKRPersistentChangeTokenPayload) throws -> PHPersistentChangeToken {
    guard let data = Data(base64Encoded: payload.dataBase64),
          let token = try NSKeyedUnarchiver.unarchivedObject(ofClass: PHPersistentChangeToken.self, from: data)
    else {
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "invalid persistent change token"])
    }
    return token
}

func pkrEncodePersistentChange(_ change: PHPersistentChange) throws -> PKRPersistentChangePayload {
    var details: [PKRPersistentObjectChangeDetailsPayload] = []
    for objectType in [PHObjectType.asset, .assetCollection, .collectionList] {
        if let objectDetails = try? change.changeDetails(for: objectType) {
            details.append(PKRPersistentObjectChangeDetailsPayload(
                objectType: objectDetails.objectType.rawValue,
                insertedLocalIdentifiers: Array(objectDetails.insertedLocalIdentifiers).sorted(),
                updatedLocalIdentifiers: Array(objectDetails.updatedLocalIdentifiers).sorted(),
                deletedLocalIdentifiers: Array(objectDetails.deletedLocalIdentifiers).sorted()
            ))
        }
    }
    return PKRPersistentChangePayload(changeToken: try pkrEncodePersistentChangeToken(change.changeToken), changeDetails: details)
}

final class PKRChangeObserverBox: NSObject, PHPhotoLibraryChangeObserver {
    let library: PHPhotoLibrary
    let callback: PKRChangeObserverCallback
    let userInfo: UnsafeMutableRawPointer?

    init(
        library: PHPhotoLibrary,
        callback: @escaping PKRChangeObserverCallback,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.library = library
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    func photoLibraryDidChange(_ changeInstance: PHChange) {
        callback(pkrRetain(PKRChangeBox(change: changeInstance)), userInfo)
    }
}

func pkrAccessLevel(rawValue: Int32) throws -> PHAccessLevel {
    switch rawValue {
    case 1:
        return .addOnly
    case 2:
        return .readWrite
    default:
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHAccessLevel raw value: \(rawValue)"]
        )
    }
}

@_cdecl("ph_authorization_status")
public func ph_authorization_status() -> Int32 {
    if #available(macOS 11.0, *) {
        return Int32(PHPhotoLibrary.authorizationStatus(for: .readWrite).rawValue)
    }
    return Int32(PHPhotoLibrary.authorizationStatus().rawValue)
}

@_cdecl("ph_request_authorization")
public func ph_request_authorization(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    ph_request_authorization_for_access_level(2, outError)
}

@_cdecl("ph_authorization_status_for_access_level")
public func ph_authorization_status_for_access_level(_ accessLevelRawValue: Int32) -> Int32 {
    if #available(macOS 11.0, *) {
        do {
            return Int32(PHPhotoLibrary.authorizationStatus(for: try pkrAccessLevel(rawValue: accessLevelRawValue)).rawValue)
        } catch {
            return Int32(PHPhotoLibrary.authorizationStatus(for: .readWrite).rawValue)
        }
    }
    return Int32(PHPhotoLibrary.authorizationStatus().rawValue)
}

@_cdecl("ph_request_authorization_for_access_level")
public func ph_request_authorization_for_access_level(
    _ accessLevelRawValue: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var status: PHAuthorizationStatus = .notDetermined

    if #available(macOS 11.0, *) {
        do {
            PHPhotoLibrary.requestAuthorization(for: try pkrAccessLevel(rawValue: accessLevelRawValue)) {
                status = $0
                semaphore.signal()
            }
        } catch {
            pkrSetError(outError, error)
            return Int32(PHAuthorizationStatus.denied.rawValue)
        }
    } else {
        PHPhotoLibrary.requestAuthorization {
            status = $0
            semaphore.signal()
        }
    }

    _ = semaphore.wait(timeout: .now() + .seconds(30))
    if status == .notDetermined {
        pkrSetMessageError(outError, message: "photo authorization did not resolve")
    }
    return Int32(status.rawValue)
}

@_cdecl("ph_photo_library_shared")
public func ph_photo_library_shared() -> UnsafeMutableRawPointer {
    pkrRetain(PKRPhotoLibraryBox(library: PHPhotoLibrary.shared()))
}

@_cdecl("ph_photo_library_release")
public func ph_photo_library_release(_ library: UnsafeMutableRawPointer?) {
    guard let library else { return }
    pkrRelease(library)
}

@_cdecl("ph_photo_library_fetch_asset_collections_json")
public func ph_photo_library_fetch_asset_collections_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    ph_asset_collection_fetch_all_json(fetchOptionsJSON, outError)
}

@_cdecl("ph_photo_library_fetch_assets_json")
public func ph_photo_library_fetch_assets_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    ph_asset_fetch_all_json(fetchOptionsJSON, outError)
}

@_cdecl("ph_photo_library_register_change_observer")
public func ph_photo_library_register_change_observer(
    _ library: UnsafeMutableRawPointer?,
    _ callback: @escaping PKRChangeObserverCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
    let observer = PKRChangeObserverBox(
        library: photoLibrary,
        callback: callback,
        userInfo: userInfo
    )
    photoLibrary.register(observer)
    return pkrRetain(observer)
}

@_cdecl("ph_photo_library_unregister_change_observer")
public func ph_photo_library_unregister_change_observer(_ observer: UnsafeMutableRawPointer?) {
    guard let observer else { return }
    let box = pkrBorrow(observer, as: PKRChangeObserverBox.self)
    box.library.unregisterChangeObserver(box)
    pkrRelease(observer)
}


@_cdecl("ph_photo_library_unavailability_reason_json")
public func ph_photo_library_unavailability_reason_json(
    _ library: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }
    let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
    do {
        return pkrCString(try pkrEncodeJSON(photoLibrary.unavailabilityReason.map(pkrErrorPayload)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_photo_library_register_availability_observer")
public func ph_photo_library_register_availability_observer(
    _ library: UnsafeMutableRawPointer?,
    _ callback: @escaping PKRAvailabilityObserverCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
    let observer = PKRAvailabilityObserverBox(
        library: photoLibrary,
        callback: callback,
        userInfo: userInfo
    )
    photoLibrary.register(observer)
    return pkrRetain(observer)
}

@_cdecl("ph_photo_library_unregister_availability_observer")
public func ph_photo_library_unregister_availability_observer(_ observer: UnsafeMutableRawPointer?) {
    guard let observer else { return }
    let box = pkrBorrow(observer, as: PKRAvailabilityObserverBox.self)
    box.library.unregisterAvailabilityObserver(box)
    pkrRelease(observer)
}

@_cdecl("ph_photo_library_current_change_token_json")
public func ph_photo_library_current_change_token_json(
    _ library: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    do {
        let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
        return pkrCString(try pkrEncodeJSON(try pkrEncodePersistentChangeToken(photoLibrary.currentChangeToken)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_photo_library_fetch_persistent_changes_since_token_json")
public func ph_photo_library_fetch_persistent_changes_since_token_json(
    _ library: UnsafeMutableRawPointer?,
    _ tokenJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    do {
        let photoLibrary = pkrBorrow(library, as: PKRPhotoLibraryBox.self).library
        let tokenPayload = try pkrDecodeJSON(tokenJSON, as: PKRPersistentChangeTokenPayload.self)
        let token = try pkrDecodePersistentChangeToken(tokenPayload)
        let result = try photoLibrary.fetchPersistentChanges(since: token)
        var changes: [PKRPersistentChangePayload] = []
        for change in result {
            if let payload = try? pkrEncodePersistentChange(change) {
                changes.append(payload)
            }
        }
        return pkrCString(try pkrEncodeJSON(PKRPersistentChangeFetchResultPayload(changes: changes)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
