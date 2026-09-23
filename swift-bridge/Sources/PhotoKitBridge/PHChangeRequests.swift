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

func pkrDate(from string: String?) throws -> Date? {
    try string.map(pkrParseDate)
}

func pkrLocation(from payload: PKRCoordinatePayload?) -> CLLocation? {
    guard let payload else { return nil }
    return CLLocation(latitude: payload.latitude, longitude: payload.longitude)
}

enum PKRMutationKind: String {
    case add
    case insert
    case remove
    case removeAtIndexes
    case replace
    case move
}

struct PKRResolvedMutation<Element> {
    var kind: PKRMutationKind
    var objects: [Element]
    var indexes: IndexSet
    var toIndex: Int
}

func pkrCheckedIndexes(_ indexes: [Int], below limit: Int) throws -> IndexSet {
    guard indexes.allSatisfy({ $0 >= 0 && $0 < limit }) else {
        throw pkrError("mutation indexes \(indexes) are outside 0..<\(limit)")
    }
    let indexSet = IndexSet(indexes)
    guard indexSet.count == indexes.count else {
        throw pkrError("mutation indexes \(indexes) contain duplicates")
    }
    return indexSet
}

func pkrResolveMutation<Element>(
    kind rawKind: String,
    objects: [Element],
    indexes rawIndexes: [Int],
    toIndex rawToIndex: Int?,
    count: inout Int?
) throws -> PKRResolvedMutation<Element> {
    guard let kind = PKRMutationKind(rawValue: rawKind) else {
        throw pkrError("unsupported mutation kind: \(rawKind)")
    }
    if kind == .add || kind == .remove {
        count = nil
        return PKRResolvedMutation(kind: kind, objects: objects, indexes: IndexSet(), toIndex: 0)
    }
    guard let current = count else {
        throw pkrError("index-based mutations cannot follow add or remove in the same change request")
    }
    if (kind == .insert || kind == .replace) && objects.count != rawIndexes.count {
        throw pkrError("\(rawKind) needs exactly one index per object")
    }
    let limit = kind == .insert ? current + objects.count : current
    let indexes = try pkrCheckedIndexes(rawIndexes, below: limit)
    var toIndex = 0
    switch kind {
    case .insert:
        count = limit
    case .removeAtIndexes:
        count = current - indexes.count
    case .move:
        guard let destination = rawToIndex, destination >= 0, destination <= current - indexes.count else {
            throw pkrError("move destination index is outside 0...\(current - indexes.count)")
        }
        toIndex = destination
    case .add, .remove, .replace:
        break
    }
    return PKRResolvedMutation(kind: kind, objects: objects, indexes: indexes, toIndex: toIndex)
}

func pkrPerformChangesAndWait(_ change: @escaping () throws -> String?) throws -> String? {
    var placeholder: String?
    var changeError: Error?
    try PHPhotoLibrary.shared().performChangesAndWait {
        do {
            placeholder = try change()
        } catch {
            changeError = error
        }
    }
    if let changeError {
        throw changeError
    }
    return placeholder
}

enum PKRAssetChangeSource {
    case existing(PHAsset)
    case imageFile(URL)
    case imageData(NSImage)
    case videoFile(URL)
}

struct PKRResolvedAssetChange {
    var source: PKRAssetChangeSource
    var creationDate: Date?
    var payload: PKRAssetChangeRequestPayload
}

func pkrResolveAssetChange(_ payload: PKRAssetChangeRequestPayload) throws -> PKRResolvedAssetChange {
    let source: PKRAssetChangeSource
    if let identifier = payload.assetLocalIdentifier {
        source = .existing(try pkrRequestAsset(localIdentifier: identifier))
    } else if let imageFileURL = payload.createImageFileURL {
        source = .imageFile(try pkrReadableFileURL(imageFileURL))
    } else if let imageDataBase64 = payload.createImageDataBase64 {
        guard let data = Data(base64Encoded: imageDataBase64) else {
            throw pkrError("image data is not valid base64")
        }
        guard let image = NSImage(data: data) else {
            throw pkrError("image data is not a decodable image")
        }
        source = .imageData(image)
    } else if let videoFileURL = payload.createVideoFileURL {
        source = .videoFile(try pkrReadableFileURL(videoFileURL))
    } else {
        throw pkrError("asset change request needs an asset identifier or a creation source")
    }
    return PKRResolvedAssetChange(
        source: source,
        creationDate: try pkrDate(from: payload.setCreationDate),
        payload: payload
    )
}

func pkrApplyAssetChange(_ change: PKRResolvedAssetChange) throws -> String? {
    let request: PHAssetChangeRequest
    switch change.source {
    case .existing(let asset):
        request = PHAssetChangeRequest(for: asset)
    case .imageFile(let url):
        guard let created = PHAssetChangeRequest.creationRequestForAssetFromImage(atFileURL: url) else {
            throw pkrError("cannot create an image asset from \(url.path)")
        }
        request = created
    case .imageData(let image):
        request = PHAssetChangeRequest.creationRequestForAsset(from: image)
    case .videoFile(let url):
        guard let created = PHAssetChangeRequest.creationRequestForAssetFromVideo(atFileURL: url) else {
            throw pkrError("cannot create a video asset from \(url.path)")
        }
        request = created
    }

    let payload = change.payload
    if let creationDate = change.creationDate {
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
    return request.placeholderForCreatedAsset?.localIdentifier
}

@_cdecl("ph_asset_change_request_perform_json")
public func ph_asset_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let change = try pkrResolveAssetChange(try pkrDecodeJSON(payloadJSON, as: PKRAssetChangeRequestPayload.self))
        let placeholderLocalIdentifier = try pkrPerformChangesAndWait { try pkrApplyAssetChange(change) }
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

enum PKRAssetCollectionTarget {
    case create(String)
    case existing(PHAssetCollection, PHFetchResult<PHAsset>)
}

struct PKRResolvedAssetCollectionChange {
    var target: PKRAssetCollectionTarget
    var title: String?
    var mutations: [PKRResolvedMutation<PHAsset>]
}

func pkrResolveAssetCollectionChange(_ payload: PKRAssetCollectionChangeRequestPayload) throws -> PKRResolvedAssetCollectionChange {
    let target: PKRAssetCollectionTarget
    var count: Int?
    if let creationTitle = payload.creationTitle {
        target = .create(creationTitle)
        count = 0
    } else if let identifier = payload.assetCollectionLocalIdentifier {
        let collection = try pkrRequestAssetCollection(localIdentifier: identifier)
        let assets = PHAsset.fetchAssets(in: collection, options: nil)
        target = .existing(collection, assets)
        count = assets.count
    } else {
        throw pkrError("asset collection change request needs a creation title or a collection identifier")
    }
    let mutations = try payload.assetMutations.map { mutation in
        try pkrResolveMutation(
            kind: mutation.kind,
            objects: mutation.assetLocalIdentifiers.map(pkrRequestAsset),
            indexes: mutation.indexes,
            toIndex: mutation.toIndex,
            count: &count
        )
    }
    return PKRResolvedAssetCollectionChange(target: target, title: payload.title, mutations: mutations)
}

func pkrApplyAssetCollectionChange(_ change: PKRResolvedAssetCollectionChange) throws -> String? {
    let request: PHAssetCollectionChangeRequest
    let isCreation: Bool
    switch change.target {
    case .create(let title):
        request = PHAssetCollectionChangeRequest.creationRequestForAssetCollection(withTitle: title)
        isCreation = true
    case .existing(let collection, let assets):
        guard let existing = PHAssetCollectionChangeRequest(for: collection, assets: assets) else {
            throw pkrError("asset collection cannot be edited: \(collection.localIdentifier)")
        }
        request = existing
        isCreation = false
    }
    if let title = change.title {
        request.title = title
    }
    for mutation in change.mutations {
        let assets = NSArray(array: mutation.objects)
        switch mutation.kind {
        case .add:
            request.addAssets(assets)
        case .insert:
            request.insertAssets(assets, at: mutation.indexes)
        case .remove:
            request.removeAssets(assets)
        case .removeAtIndexes:
            request.removeAssets(at: mutation.indexes)
        case .replace:
            request.replaceAssets(at: mutation.indexes, withAssets: assets)
        case .move:
            request.moveAssets(at: mutation.indexes, to: mutation.toIndex)
        }
    }
    return isCreation ? request.placeholderForCreatedAssetCollection.localIdentifier : nil
}

@_cdecl("ph_asset_collection_change_request_perform_json")
public func ph_asset_collection_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let change = try pkrResolveAssetCollectionChange(try pkrDecodeJSON(payloadJSON, as: PKRAssetCollectionChangeRequestPayload.self))
        let placeholderLocalIdentifier = try pkrPerformChangesAndWait { try pkrApplyAssetCollectionChange(change) }
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

enum PKRCollectionListTarget {
    case create(String)
    case topLevel(PHFetchResult<PHCollection>)
    case existing(PHCollectionList, PHFetchResult<PHCollection>)
}

struct PKRResolvedCollectionListChange {
    var target: PKRCollectionListTarget
    var title: String?
    var mutations: [PKRResolvedMutation<PHCollection>]
}

func pkrResolveCollectionListChange(_ payload: PKRCollectionListChangeRequestPayload) throws -> PKRResolvedCollectionListChange {
    let target: PKRCollectionListTarget
    var count: Int?
    if let creationTitle = payload.creationTitle {
        target = .create(creationTitle)
        count = 0
    } else if payload.topLevelUserCollections {
        let children = PHCollection.fetchTopLevelUserCollections(with: nil)
        target = .topLevel(children)
        count = children.count
    } else if let identifier = payload.collectionListLocalIdentifier {
        let collectionList = try pkrRequestCollectionList(localIdentifier: identifier)
        let children = PHCollection.fetchCollections(in: collectionList, options: nil)
        target = .existing(collectionList, children)
        count = children.count
    } else {
        throw pkrError("collection list change request needs a creation title, the top-level list, or a collection list identifier")
    }
    let mutations = try payload.childMutations.map { mutation in
        try pkrResolveMutation(
            kind: mutation.kind,
            objects: mutation.childLocalIdentifiers.map(pkrRequestCollection),
            indexes: mutation.indexes,
            toIndex: mutation.toIndex,
            count: &count
        )
    }
    return PKRResolvedCollectionListChange(target: target, title: payload.title, mutations: mutations)
}

func pkrApplyCollectionListChange(_ change: PKRResolvedCollectionListChange) throws -> String? {
    let request: PHCollectionListChangeRequest
    let isCreation: Bool
    switch change.target {
    case .create(let title):
        request = PHCollectionListChangeRequest.creationRequestForCollectionList(withTitle: title)
        isCreation = true
    case .topLevel(let children):
        guard let topLevel = PHCollectionListChangeRequest(forTopLevelCollectionListUserCollections: children) else {
            throw pkrError("the top-level collection list cannot be edited")
        }
        request = topLevel
        isCreation = false
    case .existing(let collectionList, let children):
        guard let existing = PHCollectionListChangeRequest(for: collectionList, childCollections: children) else {
            throw pkrError("collection list cannot be edited: \(collectionList.localIdentifier)")
        }
        request = existing
        isCreation = false
    }
    if let title = change.title {
        request.title = title
    }
    for mutation in change.mutations {
        let collections = NSArray(array: mutation.objects)
        switch mutation.kind {
        case .add:
            request.addChildCollections(collections)
        case .insert:
            request.insertChildCollections(collections, at: mutation.indexes)
        case .remove:
            request.removeChildCollections(collections)
        case .removeAtIndexes:
            request.removeChildCollections(at: mutation.indexes)
        case .replace:
            request.replaceChildCollections(at: mutation.indexes, withChildCollections: collections)
        case .move:
            request.moveChildCollections(at: mutation.indexes, to: mutation.toIndex)
        }
    }
    return isCreation ? request.placeholderForCreatedCollectionList.localIdentifier : nil
}

@_cdecl("ph_collection_list_change_request_perform_json")
public func ph_collection_list_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let change = try pkrResolveCollectionListChange(try pkrDecodeJSON(payloadJSON, as: PKRCollectionListChangeRequestPayload.self))
        let placeholderLocalIdentifier = try pkrPerformChangesAndWait { try pkrApplyCollectionListChange(change) }
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
