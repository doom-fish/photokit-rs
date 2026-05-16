import Foundation
import Photos

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

func pkrBuildFetchOptions(_ payload: PKRFetchOptionsPayload) -> PHFetchOptions {
    let options = PHFetchOptions()
    if let predicate = payload.predicate, !predicate.isEmpty {
        options.predicate = NSPredicate(format: predicate)
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
    return options
}
