import Foundation
import Photos

enum PKRCollectionListType: String, Codable {
    case folder
    case smartFolder
}

struct PKRCollectionListPayload: Codable {
    var localIdentifier: String
    var localizedTitle: String?
    var collectionListType: PKRCollectionListType
    var collectionListSubtype: Int
    var startDate: String?
    var endDate: String?
    var localizedLocationNames: [String]
    var canContainAssets: Bool
    var canContainCollections: Bool
}

func pkrCollectionListType(from collectionListType: PHCollectionListType) -> PKRCollectionListType {
    switch collectionListType {
    case .folder:
        return .folder
    case .smartFolder:
        return .smartFolder
    @unknown default:
        return .folder
    }
}

func pkrCollectionListType(rawValue: Int32) throws -> PHCollectionListType {
    switch rawValue {
    case 2:
        return .folder
    case 3:
        return .smartFolder
    default:
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "unsupported PHCollectionListType raw value: \(rawValue)"]
        )
    }
}

func pkrEncodeCollectionList(_ collectionList: PHCollectionList) -> PKRCollectionListPayload {
    PKRCollectionListPayload(
        localIdentifier: collectionList.localIdentifier,
        localizedTitle: collectionList.localizedTitle,
        collectionListType: pkrCollectionListType(from: collectionList.collectionListType),
        collectionListSubtype: collectionList.collectionListSubtype.rawValue,
        startDate: pkrDateString(collectionList.startDate),
        endDate: pkrDateString(collectionList.endDate),
        localizedLocationNames: collectionList.localizedLocationNames,
        canContainAssets: collectionList.canContainAssets,
        canContainCollections: collectionList.canContainCollections
    )
}

func pkrRequestCollectionList(localIdentifier: String) throws -> PHCollectionList {
    let result = PHCollectionList.fetchCollectionLists(withLocalIdentifiers: [localIdentifier], options: nil)
    guard let collectionList = result.firstObject else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "collection list not found: \(localIdentifier)"]
        )
    }
    return collectionList
}

func pkrRequestCollection(localIdentifier: String) throws -> PHCollection {
    if let assetCollection = PHAssetCollection.fetchAssetCollections(withLocalIdentifiers: [localIdentifier], options: nil).firstObject {
        return assetCollection
    }
    if let collectionList = PHCollectionList.fetchCollectionLists(withLocalIdentifiers: [localIdentifier], options: nil).firstObject {
        return collectionList
    }
    throw NSError(
        domain: "photokit-rs",
        code: -1,
        userInfo: [NSLocalizedDescriptionKey: "collection not found: \(localIdentifier)"]
    )
}

@_cdecl("ph_collection_list_fetch_containing_collection_json")
public func ph_collection_list_fetch_containing_collection_json(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return nil
    }

    do {
        let collection = try pkrRequestCollection(localIdentifier: String(cString: collectionIdentifier))
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHCollectionList.fetchCollectionListsContaining(
            collection,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollectionList)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_list_fetch_with_local_identifiers_json")
public func ph_collection_list_fetch_with_local_identifiers_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHCollectionList.fetchCollectionLists(
            withLocalIdentifiers: identifiers,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollectionList)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_list_fetch_with_type_json")
public func ph_collection_list_fetch_with_type_json(
    _ collectionListTypeRawValue: Int32,
    _ collectionListSubtypeRawValue: Int64,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHCollectionList.fetchCollectionLists(
            with: try pkrCollectionListType(rawValue: collectionListTypeRawValue),
            subtype: PHCollectionListSubtype(rawValue: Int(collectionListSubtypeRawValue)) ?? .any,
            options: pkrBuildFetchOptions(payload)
        )
        return pkrCString(try pkrEncodeJSON(pkrCollectFetchResult(result, transform: pkrEncodeCollectionList)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_list_can_perform_edit_operation")
public func ph_collection_list_can_perform_edit_operation(
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ operationRawValue: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return 0
    }

    do {
        let collectionList = try pkrRequestCollectionList(localIdentifier: String(cString: collectionIdentifier))
        return collectionList.canPerform(try pkrCollectionEditOperation(rawValue: operationRawValue)) ? 1 : 0
    } catch {
        pkrSetError(outError, error)
        return 0
    }
}
