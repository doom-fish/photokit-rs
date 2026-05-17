import Foundation
import Photos

struct PKRCollectionPayload: Codable {
    var localIdentifier: String
    var localizedTitle: String?
    var canContainAssets: Bool
    var canContainCollections: Bool
    var kind: String
}

func pkrEncodeCollection(_ collection: PHCollection) -> PKRCollectionPayload {
    let kind: String
    if collection is PHProject {
        kind = "project"
    } else if collection is PHAssetCollection {
        kind = "assetCollection"
    } else if collection is PHCollectionList {
        kind = "collectionList"
    } else {
        kind = "collection"
    }
    return PKRCollectionPayload(
        localIdentifier: collection.localIdentifier,
        localizedTitle: collection.localizedTitle,
        canContainAssets: collection.canContainAssets,
        canContainCollections: collection.canContainCollections,
        kind: kind
    )
}

@_cdecl("ph_collection_fetch_in_collection_list_json")
public func ph_collection_fetch_in_collection_list_json(
    _ collectionListIdentifier: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let collectionListIdentifier else {
        pkrSetMessageError(outError, message: "missing collection list identifier")
        return nil
    }

    do {
        let list = try pkrRequestCollectionList(localIdentifier: String(cString: collectionListIdentifier))
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHCollection.fetchCollections(in: list, options: pkrBuildFetchOptions(payload))
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollection)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_fetch_top_level_user_collections_json")
public func ph_collection_fetch_top_level_user_collections_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHCollection.fetchTopLevelUserCollections(with: pkrBuildFetchOptions(payload))
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollection)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_can_perform_edit_operation")
public func ph_collection_can_perform_edit_operation(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ operationRawValue: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return 0
    }

    do {
        let collection = try pkrRequestCollection(localIdentifier: String(cString: collectionIdentifier))
        return collection.canPerform(try pkrCollectionEditOperation(rawValue: operationRawValue)) ? 1 : 0
    } catch {
        pkrSetError(outError, error)
        return 0
    }
}
