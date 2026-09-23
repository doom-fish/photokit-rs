import Foundation
import Photos
import PhotoKitObjCBridge

struct PKRSortDescriptorPayload: Codable {
    var key: String
    var ascending: Bool
}

struct PKRFetchOptionsPayload: Codable {
    var predicate: String?
    var sortDescriptors: [PKRSortDescriptorPayload]
    var includeHiddenAssets: Bool
    var includeAllBurstAssets: Bool
    var includeAssetSourceTypes: UInt64?
    var fetchLimit: Int?
    var wantsIncrementalChangeDetails: Bool
}

func pkrBuildFetchOptions(_ payload: PKRFetchOptionsPayload, for entity: PKRFetchEntity) throws -> PHFetchOptions {
    let options = PHFetchOptions()
    if let predicate = payload.predicate, !predicate.isEmpty {
        var error: NSError?
        guard let parsed = PKRPredicateWithFormat(predicate, &error) else {
            throw error ?? pkrError("invalid fetch predicate: \(predicate)")
        }
        options.predicate = parsed
    }
    if !payload.sortDescriptors.isEmpty {
        options.sortDescriptors = payload.sortDescriptors.map {
            NSSortDescriptor(key: $0.key, ascending: $0.ascending)
        }
    }
    options.includeHiddenAssets = payload.includeHiddenAssets
    if #available(macOS 10.15, *) {
        options.includeAllBurstAssets = payload.includeAllBurstAssets
    }
    if let includeAssetSourceTypes = payload.includeAssetSourceTypes {
        options.includeAssetSourceTypes = PHAssetSourceType(rawValue: UInt(includeAssetSourceTypes))
    }
    if let fetchLimit = payload.fetchLimit {
        options.fetchLimit = fetchLimit
    }
    options.wantsIncrementalChangeDetails = payload.wantsIncrementalChangeDetails
    if options.predicate != nil || !payload.sortDescriptors.isEmpty {
        var error: NSError?
        guard PKRValidateFetchOptions(options, entity, &error) else {
            throw error ?? pkrError("unsupported fetch options")
        }
    }
    return options
}
