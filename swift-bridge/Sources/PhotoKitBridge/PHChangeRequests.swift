import AppKit
import CoreLocation
import Foundation
import Photos

struct PKRChangeRequestPerformResultPayload: Codable {
    var placeholderLocalIdentifier: String?
}

struct PKRAssetChangeRequestPayload: Codable {
    var assetLocalIdentifier: String?
    var createImageFileURL: String?
    var createImageDataBase64: String?
    var createVideoFileURL: String?
    var setCreationDate: String?
    var clearCreationDate: Bool
    var setLocation: PKRCoordinatePayload?
    var clearLocation: Bool
    var favorite: Bool?
    var hidden: Bool?
    var revertAssetContentToOriginal: Bool
}

struct PKRAssetCollectionAssetMutationPayload: Codable {
    var kind: String
    var assetLocalIdentifiers: [String]
    var indexes: [Int]
    var toIndex: Int?
}

struct PKRAssetCollectionChangeRequestPayload: Codable {
    var assetCollectionLocalIdentifier: String?
    var creationTitle: String?
    var title: String?
    var assetMutations: [PKRAssetCollectionAssetMutationPayload]
}

struct PKRCollectionListChildMutationPayload: Codable {
    var kind: String
    var childLocalIdentifiers: [String]
    var indexes: [Int]
    var toIndex: Int?
}

struct PKRCollectionListChangeRequestPayload: Codable {
    var collectionListLocalIdentifier: String?
    var topLevelUserCollections: Bool
    var creationTitle: String?
    var title: String?
    var childMutations: [PKRCollectionListChildMutationPayload]
}

func pkrDate(from string: String?) -> Date? {
    guard let string else { return nil }
    return ISO8601DateFormatter().date(from: string)
}

func pkrLocation(from payload: PKRCoordinatePayload?) -> CLLocation? {
    guard let payload else { return nil }
    return CLLocation(latitude: payload.latitude, longitude: payload.longitude)
}

func pkrIndexSet(_ indexes: [Int]) -> IndexSet {
    IndexSet(indexes)
}

@_cdecl("ph_asset_change_request_perform_json")
public func ph_asset_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(payloadJSON, as: PKRAssetChangeRequestPayload.self)
        var placeholderLocalIdentifier: String?
        try PHPhotoLibrary.shared().performChangesAndWait {
            let request: PHAssetChangeRequest
            if let assetLocalIdentifier = payload.assetLocalIdentifier {
                request = PHAssetChangeRequest(for: try! pkrRequestAsset(localIdentifier: assetLocalIdentifier))
            } else if let imageFileURL = payload.createImageFileURL {
                guard let created = PHAssetChangeRequest.creationRequestForAssetFromImage(atFileURL: pkrAssetCreationURL(imageFileURL)) else {
                    fatalError("failed to create image asset change request")
                }
                request = created
            } else if let imageDataBase64 = payload.createImageDataBase64,
                      let data = Data(base64Encoded: imageDataBase64),
                      let image = NSImage(data: data) {
                request = PHAssetChangeRequest.creationRequestForAsset(from: image)
            } else if let videoFileURL = payload.createVideoFileURL,
                      let created = PHAssetChangeRequest.creationRequestForAssetFromVideo(atFileURL: pkrAssetCreationURL(videoFileURL)) {
                request = created
            } else {
                fatalError("invalid asset change request payload")
            }

            if let creationDate = pkrDate(from: payload.setCreationDate) {
                request.creationDate = creationDate
            }
            if payload.clearCreationDate {
                request.creationDate = nil
            }
            if let location = pkrLocation(from: payload.setLocation) {
                request.location = location
            }
            if payload.clearLocation {
                request.location = nil
            }
            if let favorite = payload.favorite {
                request.isFavorite = favorite
            }
            if let hidden = payload.hidden {
                request.isHidden = hidden
            }
            if payload.revertAssetContentToOriginal {
                request.revertAssetContentToOriginal()
            }
            placeholderLocalIdentifier = request.placeholderForCreatedAsset?.localIdentifier
        }
        return pkrCString(try pkrEncodeJSON(PKRChangeRequestPerformResultPayload(placeholderLocalIdentifier: placeholderLocalIdentifier)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_change_request_delete_assets_json")
public func ph_asset_change_request_delete_assets_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let assets = try identifiers.map(pkrRequestAsset)
        try PHPhotoLibrary.shared().performChangesAndWait {
            PHAssetChangeRequest.deleteAssets(assets as NSFastEnumeration)
        }
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

func pkrApplyAssetCollectionMutation(_ mutation: PKRAssetCollectionAssetMutationPayload, to request: PHAssetCollectionChangeRequest) throws {
    switch mutation.kind {
    case "add":
        request.addAssets(NSArray(array: try mutation.assetLocalIdentifiers.map(pkrRequestAsset)))
    case "insert":
        request.insertAssets(NSArray(array: try mutation.assetLocalIdentifiers.map(pkrRequestAsset)), at: pkrIndexSet(mutation.indexes))
    case "remove":
        request.removeAssets(NSArray(array: try mutation.assetLocalIdentifiers.map(pkrRequestAsset)))
    case "removeAtIndexes":
        request.removeAssets(at: pkrIndexSet(mutation.indexes))
    case "replace":
        request.replaceAssets(at: pkrIndexSet(mutation.indexes), withAssets: NSArray(array: try mutation.assetLocalIdentifiers.map(pkrRequestAsset)))
    case "move":
        request.moveAssets(at: pkrIndexSet(mutation.indexes), to: mutation.toIndex ?? 0)
    default:
        break
    }
}

@_cdecl("ph_asset_collection_change_request_perform_json")
public func ph_asset_collection_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(payloadJSON, as: PKRAssetCollectionChangeRequestPayload.self)
        var placeholderLocalIdentifier: String?
        try PHPhotoLibrary.shared().performChangesAndWait {
            let request: PHAssetCollectionChangeRequest
            if let creationTitle = payload.creationTitle {
                request = PHAssetCollectionChangeRequest.creationRequestForAssetCollection(withTitle: creationTitle)
            } else if let identifier = payload.assetCollectionLocalIdentifier,
                      let existing = PHAssetCollectionChangeRequest(for: try! pkrRequestAssetCollection(localIdentifier: identifier)) {
                request = existing
            } else {
                fatalError("invalid asset collection change request payload")
            }
            if let title = payload.title {
                request.title = title
            }
            for mutation in payload.assetMutations {
                try! pkrApplyAssetCollectionMutation(mutation, to: request)
            }
            placeholderLocalIdentifier = request.placeholderForCreatedAssetCollection.localIdentifier
        }
        return pkrCString(try pkrEncodeJSON(PKRChangeRequestPerformResultPayload(placeholderLocalIdentifier: placeholderLocalIdentifier)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_collection_change_request_delete_json")
public func ph_asset_collection_change_request_delete_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let collections = try identifiers.map(pkrRequestAssetCollection)
        try PHPhotoLibrary.shared().performChangesAndWait {
            PHAssetCollectionChangeRequest.deleteAssetCollections(collections as NSFastEnumeration)
        }
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

func pkrApplyCollectionListMutation(_ mutation: PKRCollectionListChildMutationPayload, to request: PHCollectionListChangeRequest) throws {
    switch mutation.kind {
    case "add":
        request.addChildCollections(NSArray(array: try mutation.childLocalIdentifiers.map(pkrRequestCollection)))
    case "insert":
        request.insertChildCollections(NSArray(array: try mutation.childLocalIdentifiers.map(pkrRequestCollection)), at: pkrIndexSet(mutation.indexes))
    case "remove":
        request.removeChildCollections(NSArray(array: try mutation.childLocalIdentifiers.map(pkrRequestCollection)))
    case "removeAtIndexes":
        request.removeChildCollections(at: pkrIndexSet(mutation.indexes))
    case "replace":
        request.replaceChildCollections(at: pkrIndexSet(mutation.indexes), withChildCollections: NSArray(array: try mutation.childLocalIdentifiers.map(pkrRequestCollection)))
    case "move":
        request.moveChildCollections(at: pkrIndexSet(mutation.indexes), to: mutation.toIndex ?? 0)
    default:
        break
    }
}

@_cdecl("ph_collection_list_change_request_perform_json")
public func ph_collection_list_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(payloadJSON, as: PKRCollectionListChangeRequestPayload.self)
        var placeholderLocalIdentifier: String?
        try PHPhotoLibrary.shared().performChangesAndWait {
            let request: PHCollectionListChangeRequest
            if let creationTitle = payload.creationTitle {
                request = PHCollectionListChangeRequest.creationRequestForCollectionList(withTitle: creationTitle)
            } else if payload.topLevelUserCollections {
                let result = PHCollection.fetchTopLevelUserCollections(with: nil)
                guard let topLevel = PHCollectionListChangeRequest(forTopLevelCollectionListUserCollections: result) else {
                    fatalError("failed to create top-level collection list change request")
                }
                request = topLevel
            } else if let identifier = payload.collectionListLocalIdentifier,
                      let existing = PHCollectionListChangeRequest(for: try! pkrRequestCollectionList(localIdentifier: identifier)) {
                request = existing
            } else {
                fatalError("invalid collection list change request payload")
            }
            if let title = payload.title {
                request.title = title
            }
            for mutation in payload.childMutations {
                try! pkrApplyCollectionListMutation(mutation, to: request)
            }
            placeholderLocalIdentifier = request.placeholderForCreatedCollectionList.localIdentifier
        }
        return pkrCString(try pkrEncodeJSON(PKRChangeRequestPerformResultPayload(placeholderLocalIdentifier: placeholderLocalIdentifier)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_collection_list_change_request_delete_json")
public func ph_collection_list_change_request_delete_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        let collectionLists = try identifiers.map(pkrRequestCollectionList)
        try PHPhotoLibrary.shared().performChangesAndWait {
            PHCollectionListChangeRequest.deleteCollectionLists(collectionLists as NSFastEnumeration)
        }
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}
