import CoreLocation
import Foundation
import Photos

enum PKRMediaType: String, Codable {
    case unknown
    case image
    case video
    case audio
}

enum PKRAssetPlaybackStyle: String, Codable {
    case unsupported
    case image
    case imageAnimated
    case livePhoto
    case video
    case videoLooping
}

struct PKRCoordinatePayload: Codable {
    var latitude: Double
    var longitude: Double
}

struct PKRAssetPayload: Codable {
    var localIdentifier: String
    var creationDate: String?
    var modificationDate: String?
    var addedDate: String?
    var pixelWidth: UInt64
    var pixelHeight: UInt64
    var location: PKRCoordinatePayload?
    var mediaType: PKRMediaType
    var mediaSubtypes: UInt64
    var duration: Double
    var isHidden: Bool
    var isFavorite: Bool
    var playbackStyle: PKRAssetPlaybackStyle?
    var contentTypeIdentifier: String?
    var burstIdentifier: String?
    var burstSelectionTypes: UInt64
    var representsBurst: Bool
    var sourceType: UInt64
    var hasAdjustments: Bool
    var adjustmentFormatIdentifier: String?
}

struct PKRAssetResourcePayload: Codable {
    var assetLocalIdentifier: String
    var resourceType: Int
    var originalFilename: String
    var uniformTypeIdentifier: String?
    var contentTypeIdentifier: String?
    var pixelWidth: Int?
    var pixelHeight: Int?
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

func pkrAssetPlaybackStyle(from playbackStyle: PHAsset.PlaybackStyle) -> PKRAssetPlaybackStyle {
    switch playbackStyle {
    case .unsupported:
        return .unsupported
    case .image:
        return .image
    case .imageAnimated:
        return .imageAnimated
    case .livePhoto:
        return .livePhoto
    case .video:
        return .video
    case .videoLooping:
        return .videoLooping
    @unknown default:
        return .unsupported
    }
}

func pkrCoordinatePayload(_ location: CLLocation?) -> PKRCoordinatePayload? {
    guard let location else { return nil }
    return PKRCoordinatePayload(
        latitude: location.coordinate.latitude,
        longitude: location.coordinate.longitude
    )
}

func pkrEncodeAsset(_ asset: PHAsset) -> PKRAssetPayload {
    PKRAssetPayload(
        localIdentifier: asset.localIdentifier,
        creationDate: pkrDateString(asset.creationDate),
        modificationDate: pkrDateString(asset.modificationDate),
        addedDate: {
            if #available(macOS 26.0, *) {
                return pkrDateString(asset.addedDate)
            }
            return nil
        }(),
        pixelWidth: UInt64(asset.pixelWidth),
        pixelHeight: UInt64(asset.pixelHeight),
        location: pkrCoordinatePayload(asset.location),
        mediaType: pkrMediaType(from: asset.mediaType),
        mediaSubtypes: UInt64(asset.mediaSubtypes.rawValue),
        duration: asset.duration,
        isHidden: asset.isHidden,
        isFavorite: asset.isFavorite,
        playbackStyle: {
            if #available(macOS 10.15, *) {
                return pkrAssetPlaybackStyle(from: asset.playbackStyle)
            }
            return nil
        }(),
        contentTypeIdentifier: {
            if #available(macOS 26.0, *) {
                return asset.contentType.identifier
            }
            return nil
        }(),
        burstIdentifier: {
            if #available(macOS 10.15, *) {
                return asset.burstIdentifier
            }
            return nil
        }(),
        burstSelectionTypes: {
            if #available(macOS 10.15, *) {
                return UInt64(asset.burstSelectionTypes.rawValue)
            }
            return 0
        }(),
        representsBurst: {
            if #available(macOS 10.15, *) {
                return asset.representsBurst
            }
            return false
        }(),
        sourceType: UInt64(asset.sourceType.rawValue),
        hasAdjustments: {
            if #available(macOS 12.0, *) {
                return asset.hasAdjustments
            }
            return false
        }(),
        adjustmentFormatIdentifier: {
            if #available(macOS 12.0, *) {
                return asset.adjustmentFormatIdentifier
            }
            return nil
        }()
    )
}

func pkrEncodeResource(_ resource: PHAssetResource) -> PKRAssetResourcePayload {
    PKRAssetResourcePayload(
        assetLocalIdentifier: resource.assetLocalIdentifier,
        resourceType: resource.type.rawValue,
        originalFilename: resource.originalFilename,
        uniformTypeIdentifier: resource.uniformTypeIdentifier,
        contentTypeIdentifier: {
            if #available(macOS 26.0, *) {
                return resource.contentType.identifier
            }
            return nil
        }(),
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

func pkrAssetMediaType(rawValue: Int32) throws -> PHAssetMediaType {
    switch rawValue {
    case 0:
        return .unknown
    case 1:
        return .image
    case 2:
        return .video
    case 3:
        return .audio
    default:
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHAssetMediaType raw value: \(rawValue)"]
        )
    }
}

func pkrAssetEditOperation(rawValue: Int32) throws -> PHAssetEditOperation {
    guard let operation = PHAssetEditOperation(rawValue: Int(rawValue)) else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHAssetEditOperation raw value: \(rawValue)"]
        )
    }
    return operation
}

@_cdecl("ph_asset_json")
public func ph_asset_json(
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        return pkrCString(try pkrEncodeJSON(pkrEncodeAsset(asset)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_fetch_all_json")
public func ph_asset_fetch_all_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchAssets(with: pkrBuildFetchOptions(payload))
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeAsset)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_fetch_with_media_type_json")
public func ph_asset_fetch_with_media_type_json(
    _ mediaType: Int32,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchAssets(
            with: try pkrAssetMediaType(rawValue: mediaType),
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeAsset)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_fetch_with_local_identifiers_json")
public func ph_asset_fetch_with_local_identifiers_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchAssets(
            withLocalIdentifiers: identifiers,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeAsset)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_fetch_in_collection_json")
public func ph_asset_fetch_in_collection_json(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return nil
    }

    do {
        let collection = try pkrRequestAssetCollection(localIdentifier: String(cString: collectionIdentifier))
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchAssets(in: collection, options: pkrBuildFetchOptions(payload))
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeAsset)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_fetch_key_assets_in_collection_json")
public func ph_asset_fetch_key_assets_in_collection_json(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return nil
    }

    do {
        let collection = try pkrRequestAssetCollection(localIdentifier: String(cString: collectionIdentifier))
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAsset.fetchKeyAssets(in: collection, options: pkrBuildFetchOptions(payload))
        let assets = result.map { pkrCollectFetchResult($0, transform: pkrEncodeAsset) } ?? []
        return pkrCString(try pkrEncodeJSON(assets))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_can_perform_edit_operation")
public func ph_asset_can_perform_edit_operation(
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ operationRawValue: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return 0
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        return asset.canPerform(try pkrAssetEditOperation(rawValue: operationRawValue)) ? 1 : 0
    } catch {
        pkrSetError(outError, error)
        return 0
    }
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
