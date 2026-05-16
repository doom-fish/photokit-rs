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
