import Foundation
import Photos

struct PKRAssetCollectionPayload: Codable {
    var localIdentifier: String
    var localizedTitle: String?
    var collectionType: PKRAssetCollectionType
    var collectionSubtype: Int
    var estimatedAssetCount: UInt64?
    var startDate: String?
    var endDate: String?
    var approximateLocation: PKRCoordinatePayload?
    var localizedLocationNames: [String]
    var canContainAssets: Bool
    var canContainCollections: Bool
}

enum PKRAssetCollectionType: String, Codable {
    case album
    case smartAlbum
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

func pkrAssetCollectionType(rawValue: Int32) throws -> PHAssetCollectionType {
    switch rawValue {
    case 1:
        return .album
    case 2:
        return .smartAlbum
    default:
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHAssetCollectionType raw value: \(rawValue)"]
        )
    }
}

func pkrEncodeCollection(_ collection: PHAssetCollection) -> PKRAssetCollectionPayload {
    let estimatedAssetCount: UInt64? = collection.estimatedAssetCount == NSNotFound ? nil : UInt64(collection.estimatedAssetCount)
    return PKRAssetCollectionPayload(
        localIdentifier: collection.localIdentifier,
        localizedTitle: collection.localizedTitle,
        collectionType: pkrCollectionType(from: collection.assetCollectionType),
        collectionSubtype: collection.assetCollectionSubtype.rawValue,
        estimatedAssetCount: estimatedAssetCount,
        startDate: pkrDateString(collection.startDate),
        endDate: pkrDateString(collection.endDate),
        approximateLocation: pkrCoordinatePayload(collection.approximateLocation),
        localizedLocationNames: collection.localizedLocationNames,
        canContainAssets: collection.canContainAssets,
        canContainCollections: collection.canContainCollections
    )
}

func pkrRequestAssetCollection(localIdentifier: String) throws -> PHAssetCollection {
    let result = PHAssetCollection.fetchAssetCollections(withLocalIdentifiers: [localIdentifier], options: nil)
    guard let collection = result.firstObject else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "asset collection not found: \(localIdentifier)"]
        )
    }
    return collection
}

func pkrCollectionEditOperation(rawValue: Int32) throws -> PHCollectionEditOperation {
    guard let operation = PHCollectionEditOperation(rawValue: Int(rawValue)) else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHCollectionEditOperation raw value: \(rawValue)"]
        )
    }
    return operation
}

@_cdecl("ph_asset_collection_fetch_all_json")
public func ph_asset_collection_fetch_all_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let options = pkrBuildFetchOptions(payload)
        var collections = pkrCollectFetchResult(
            PHAssetCollection.fetchAssetCollections(with: .album, subtype: .any, options: options),
            transform: pkrEncodeCollection
        )
        collections += pkrCollectFetchResult(
            PHAssetCollection.fetchAssetCollections(with: .smartAlbum, subtype: .any, options: options),
            transform: pkrEncodeCollection
        )
        return pkrCString(try pkrEncodeJSON(collections))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_collection_fetch_with_type_json")
public func ph_asset_collection_fetch_with_type_json(
    _ collectionTypeRawValue: Int32,
    _ collectionSubtypeRawValue: Int64,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAssetCollection.fetchAssetCollections(
            with: try pkrAssetCollectionType(rawValue: collectionTypeRawValue),
            subtype: PHAssetCollectionSubtype(rawValue: Int(collectionSubtypeRawValue)) ?? .any,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollection)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_collection_fetch_with_local_identifiers_json")
public func ph_asset_collection_fetch_with_local_identifiers_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAssetCollection.fetchAssetCollections(
            withLocalIdentifiers: identifiers,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollection)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_collection_fetch_containing_asset_json")
public func ph_asset_collection_fetch_containing_asset_json(
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ collectionTypeRawValue: Int32,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAssetCollection.fetchAssetCollectionsContaining(
            asset,
            with: try pkrAssetCollectionType(rawValue: collectionTypeRawValue),
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollection)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_collection_can_perform_edit_operation")
public func ph_asset_collection_can_perform_edit_operation(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ operationRawValue: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return 0
    }

    do {
        let collection = try pkrRequestAssetCollection(localIdentifier: String(cString: collectionIdentifier))
        return collection.canPerform(try pkrCollectionEditOperation(rawValue: operationRawValue)) ? 1 : 0
    } catch {
        pkrSetError(outError, error)
        return 0
    }
}
