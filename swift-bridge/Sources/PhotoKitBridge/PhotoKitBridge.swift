import AppKit
import CoreLocation
import Foundation
import Photos

enum PKRMediaType: String, Codable {
    case unknown
    case image
    case video
    case audio
}

enum PKRAssetCollectionType: String, Codable {
    case album
    case smartAlbum
}

enum PKRImageContentMode: String, Codable {
    case `default`
    case aspectFit
    case aspectFill
}

struct PKRSortDescriptorPayload: Codable {
    var key: String
    var ascending: Bool
}

struct PKRFetchOptionsPayload: Codable {
    var predicate: String?
    var sortDescriptors: [PKRSortDescriptorPayload]
    var fetchLimit: Int?
}

struct PKRCoordinatePayload: Codable {
    var latitude: Double
    var longitude: Double
}

struct PKRAssetPayload: Codable {
    var localIdentifier: String
    var creationDate: String?
    var modificationDate: String?
    var pixelWidth: UInt64
    var pixelHeight: UInt64
    var location: PKRCoordinatePayload?
    var mediaType: PKRMediaType
    var mediaSubtypes: UInt64
    var duration: Double
    var isFavorite: Bool
}

struct PKRAssetCollectionPayload: Codable {
    var localIdentifier: String
    var localizedTitle: String?
    var collectionType: PKRAssetCollectionType
    var collectionSubtype: Int
    var estimatedAssetCount: UInt64?
}

struct PKRAssetResourcePayload: Codable {
    var assetLocalIdentifier: String
    var resourceType: Int
    var originalFilename: String
    var uniformTypeIdentifier: String?
    var pixelWidth: Int?
    var pixelHeight: Int?
}

struct PKRImageRequestPayload: Codable {
    var targetWidth: Double
    var targetHeight: Double
    var contentMode: PKRImageContentMode
}

struct PKRImageResultPayload: Codable {
    var tiffDataBase64: String
    var width: Double
    var height: Double
    var cancelled: Bool
    var degraded: Bool
}

struct PKRImageDataResultPayload: Codable {
    var dataBase64: String
    var uniformTypeIdentifier: String?
    var orientation: Int32
    var cancelled: Bool
}

struct PKRLivePhotoResultPayload: Codable {
    var hasLivePhoto: Bool
    var cancelled: Bool
    var degraded: Bool
}

func pkrMediaType(from mediaType: PHAssetMediaType) -> PKRMediaType {
    switch mediaType {
    case .image:
        return .image
    case .video:
        return .video
    case .audio:
        return .audio
    case .unknown:
        return .unknown
    @unknown default:
        return .unknown
    }
}

func pkrCollectionType(from collectionType: PHAssetCollectionType) -> PKRAssetCollectionType {
    switch collectionType {
    case .album:
        return .album
    case .smartAlbum:
        return .smartAlbum
    @unknown default:
        return .album
    }
}

func pkrContentMode(from contentMode: PKRImageContentMode) -> PHImageContentMode {
    switch contentMode {
    case .default:
        return .default
    case .aspectFit:
        return .aspectFit
    case .aspectFill:
        return .aspectFill
    }
}

func pkrBuildFetchOptions(_ payload: PKRFetchOptionsPayload) -> PHFetchOptions {
    let options = PHFetchOptions()
    if let predicate = payload.predicate {
        options.predicate = NSPredicate(format: predicate)
    }
    if !payload.sortDescriptors.isEmpty {
        options.sortDescriptors = payload.sortDescriptors.map {
            NSSortDescriptor(key: $0.key, ascending: $0.ascending)
        }
    }
    if let fetchLimit = payload.fetchLimit {
        options.fetchLimit = fetchLimit
    }
    return options
}

func pkrEncodeAsset(_ asset: PHAsset) -> PKRAssetPayload {
    PKRAssetPayload(
        localIdentifier: asset.localIdentifier,
        creationDate: pkrDateString(asset.creationDate),
        modificationDate: pkrDateString(asset.modificationDate),
        pixelWidth: UInt64(asset.pixelWidth),
        pixelHeight: UInt64(asset.pixelHeight),
        location: asset.location.map { PKRCoordinatePayload(latitude: $0.coordinate.latitude, longitude: $0.coordinate.longitude) },
        mediaType: pkrMediaType(from: asset.mediaType),
        mediaSubtypes: UInt64(asset.mediaSubtypes.rawValue),
        duration: asset.duration,
        isFavorite: asset.isFavorite
    )
}

func pkrEncodeCollection(_ collection: PHAssetCollection) -> PKRAssetCollectionPayload {
    let estimatedAssetCount: UInt64? = collection.estimatedAssetCount == NSNotFound ? nil : UInt64(collection.estimatedAssetCount)
    return PKRAssetCollectionPayload(
        localIdentifier: collection.localIdentifier,
        localizedTitle: collection.localizedTitle,
        collectionType: pkrCollectionType(from: collection.assetCollectionType),
        collectionSubtype: collection.assetCollectionSubtype.rawValue,
        estimatedAssetCount: estimatedAssetCount
    )
}

func pkrEncodeResource(_ resource: PHAssetResource) -> PKRAssetResourcePayload {
    PKRAssetResourcePayload(
        assetLocalIdentifier: resource.assetLocalIdentifier,
        resourceType: resource.type.rawValue,
        originalFilename: resource.originalFilename,
        uniformTypeIdentifier: resource.uniformTypeIdentifier,
        pixelWidth: resource.responds(to: Selector(("pixelWidth"))) ? resource.pixelWidth : nil,
        pixelHeight: resource.responds(to: Selector(("pixelHeight"))) ? resource.pixelHeight : nil
    )
}

func pkrRequestAsset(localIdentifier: String) throws -> PHAsset {
    let result = PHAsset.fetchAssets(withLocalIdentifiers: [localIdentifier], options: nil)
    guard let asset = result.firstObject else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "asset not found: \(localIdentifier)"]
        )
    }
    return asset
}

@objcMembers
final class PKRRequestBox: NSObject {
    let manager: PHImageManager
    var requestID: PHImageRequestID = PHInvalidImageRequestID
    var payloadJSON: String?
    var error: Error?
    var completed = false
    let semaphore = DispatchSemaphore(value: 0)

    init(manager: PHImageManager) {
        self.manager = manager
        super.init()
    }

    func finish<T: Encodable>(_ payload: T) {
        guard !completed else { return }
        payloadJSON = try? pkrEncodeJSON(payload)
        completed = true
        semaphore.signal()
    }

    func fail(_ error: Error) {
        guard !completed else { return }
        self.error = error
        completed = true
        semaphore.signal()
    }

    func cancel(with payloadJSON: String?) {
        manager.cancelImageRequest(requestID)
        guard !completed else { return }
        self.payloadJSON = payloadJSON
        completed = true
        semaphore.signal()
    }
}

final class PKRChangeObserverBox: NSObject, PHPhotoLibraryChangeObserver {
    let library: PHPhotoLibrary
    let callback: @convention(c) (UnsafeMutableRawPointer?) -> Void
    let userInfo: UnsafeMutableRawPointer?

    init(
        library: PHPhotoLibrary,
        callback: @escaping @convention(c) (UnsafeMutableRawPointer?) -> Void,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.library = library
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    func photoLibraryDidChange(_ changeInstance: PHChange) {
        callback(userInfo)
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
    let semaphore = DispatchSemaphore(value: 0)
    var status: PHAuthorizationStatus = .notDetermined

    if #available(macOS 11.0, *) {
        PHPhotoLibrary.requestAuthorization(for: .readWrite) {
            status = $0
            semaphore.signal()
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
    pkrRetain(PHPhotoLibrary.shared())
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
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAssetCollection.fetchAssetCollections(with: .album, subtype: .any, options: pkrBuildFetchOptions(payload))
        var collections: [PKRAssetCollectionPayload] = []
        result.enumerateObjects { collection, _, _ in
            collections.append(pkrEncodeCollection(collection))
        }
        let smartAlbums = PHAssetCollection.fetchAssetCollections(with: .smartAlbum, subtype: .any, options: pkrBuildFetchOptions(payload))
        smartAlbums.enumerateObjects { collection, _, _ in
            collections.append(pkrEncodeCollection(collection))
        }
        return pkrCString(try pkrEncodeJSON(collections))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_photo_library_fetch_assets_json")
public func ph_photo_library_fetch_assets_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchAssets(with: pkrBuildFetchOptions(payload))
        var assets: [PKRAssetPayload] = []
        result.enumerateObjects { asset, _, _ in
            assets.append(pkrEncodeAsset(asset))
        }
        return pkrCString(try pkrEncodeJSON(assets))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_photo_library_register_change_observer")
public func ph_photo_library_register_change_observer(
    _ library: UnsafeMutableRawPointer?,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?) -> Void,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let library else {
        pkrSetMessageError(outError, message: "missing PHPhotoLibrary")
        return nil
    }

    let photoLibrary = pkrBorrow(library, as: PHPhotoLibrary.self)
    let observer = PKRChangeObserverBox(library: photoLibrary, callback: callback, userInfo: userInfo)
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

@_cdecl("ph_asset_resources_json")
public func ph_asset_resources_json(
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let resources = PHAssetResource.assetResources(for: asset).map(pkrEncodeResource)
        return pkrCString(try pkrEncodeJSON(resources))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_default")
public func ph_image_manager_default() -> UnsafeMutableRawPointer {
    pkrRetain(PHImageManager.default())
}

@_cdecl("ph_caching_image_manager_new")
public func ph_caching_image_manager_new() -> UnsafeMutableRawPointer {
    pkrRetain(PHCachingImageManager())
}

@_cdecl("ph_image_manager_release")
public func ph_image_manager_release(_ manager: UnsafeMutableRawPointer?) {
    guard let manager else { return }
    pkrRelease(manager)
}

@_cdecl("ph_image_manager_request_image")
public func ph_image_manager_request_image(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PHImageManager.self)
        let box = PKRRequestBox(manager: imageManager)
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        box.requestID = imageManager.requestImage(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: nil
        ) { image, info in
            let info = info ?? [:]
            let cancelled = (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false
            let degraded = (info[PHImageResultIsDegradedKey] as? NSNumber)?.boolValue ?? false
            guard let image, let data = image.tiffRepresentation else {
                box.finish(PKRImageResultPayload(tiffDataBase64: "", width: 0, height: 0, cancelled: cancelled, degraded: degraded))
                return
            }
            box.finish(
                PKRImageResultPayload(
                    tiffDataBase64: data.base64EncodedString(),
                    width: image.size.width,
                    height: image.size.height,
                    cancelled: cancelled,
                    degraded: degraded
                )
            )
        }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_image_data")
public func ph_image_manager_request_image_data(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PHImageManager.self)
        let box = PKRRequestBox(manager: imageManager)
        box.requestID = imageManager.requestImageDataAndOrientation(for: asset, options: nil) {
            imageData, dataUTI, orientation, info in
            let info = info ?? [:]
            let cancelled = (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false
            box.finish(
                PKRImageDataResultPayload(
                    dataBase64: imageData?.base64EncodedString() ?? "",
                    uniformTypeIdentifier: dataUTI,
                    orientation: Int32(orientation.rawValue),
                    cancelled: cancelled
                )
            )
        }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_live_photo")
public func ph_image_manager_request_live_photo(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PHImageManager.self)
        let box = PKRRequestBox(manager: imageManager)
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        box.requestID = imageManager.requestLivePhoto(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: nil
        ) { livePhoto, info in
            let info = info ?? [:]
            let cancelled = (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false
            let degraded = (info[PHImageResultIsDegradedKey] as? NSNumber)?.boolValue ?? false
            box.finish(
                PKRLivePhotoResultPayload(
                    hasLivePhoto: livePhoto != nil,
                    cancelled: cancelled,
                    degraded: degraded
                )
            )
        }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_request_wait_json")
public func ph_image_request_wait_json(
    _ request: UnsafeMutableRawPointer?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let request else {
        pkrSetMessageError(outError, message: "missing request handle")
        return nil
    }

    let box = pkrBorrow(request, as: PKRRequestBox.self)
    if !box.completed {
        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        if box.semaphore.wait(timeout: timeout) == .timedOut {
            pkrSetMessageError(outError, message: "request timed out")
            return nil
        }
    }

    if let error = box.error {
        pkrSetError(outError, error)
        return nil
    }
    return box.payloadJSON.flatMap(pkrCString)
}

@_cdecl("ph_image_request_cancel")
public func ph_image_request_cancel(_ request: UnsafeMutableRawPointer?) {
    guard let request else { return }
    let box = pkrBorrow(request, as: PKRRequestBox.self)
    if box.payloadJSON == nil {
        let cancelledImage = try? pkrEncodeJSON(PKRLivePhotoResultPayload(hasLivePhoto: false, cancelled: true, degraded: false))
        box.cancel(with: cancelledImage)
    } else {
        box.cancel(with: box.payloadJSON)
    }
}

@_cdecl("ph_image_request_release")
public func ph_image_request_release(_ request: UnsafeMutableRawPointer?) {
    guard let request else { return }
    pkrRelease(request)
}

@_cdecl("ph_caching_image_manager_start_caching")
public func ph_caching_image_manager_start_caching(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifiersJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHCachingImageManager")
        return PKR_ERROR
    }

    do {
        let identifiers = try pkrDecodeJSON(assetIdentifiersJSON, as: [String].self)
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let cachingManager = pkrBorrow(manager, as: PHCachingImageManager.self)
        let assets = try identifiers.map(pkrRequestAsset)
        cachingManager.startCachingImages(
            for: assets,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode),
            options: nil
        )
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_caching_image_manager_stop_caching")
public func ph_caching_image_manager_stop_caching(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifiersJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHCachingImageManager")
        return PKR_ERROR
    }

    do {
        let identifiers = try pkrDecodeJSON(assetIdentifiersJSON, as: [String].self)
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let cachingManager = pkrBorrow(manager, as: PHCachingImageManager.self)
        let assets = try identifiers.map(pkrRequestAsset)
        cachingManager.stopCachingImages(
            for: assets,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode),
            options: nil
        )
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_caching_image_manager_stop_caching_all")
public func ph_caching_image_manager_stop_caching_all(_ manager: UnsafeMutableRawPointer?) {
    guard let manager else { return }
    let cachingManager = pkrBorrow(manager, as: PHCachingImageManager.self)
    cachingManager.stopCachingImagesForAllAssets()
}
