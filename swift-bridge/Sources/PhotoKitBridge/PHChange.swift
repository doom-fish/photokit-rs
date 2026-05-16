import Foundation
import Photos

final class PKRChangeBox: NSObject {
    let change: PHChange

    init(change: PHChange) {
        self.change = change
        super.init()
    }
}

@_cdecl("ph_change_release")
public func ph_change_release(_ change: UnsafeMutableRawPointer?) {
    guard let change else { return }
    pkrRelease(change)
}

@_cdecl("ph_change_asset_change_details_json")
public func ph_change_asset_change_details_json(
    _ change: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let change else {
        pkrSetMessageError(outError, message: "missing PHChange")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let changeBox = pkrBorrow(change, as: PKRChangeBox.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let payload = changeBox.change.changeDetails(for: asset).map {
            pkrEncodeObjectChangeDetails($0, transform: pkrEncodeAsset)
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_change_asset_collection_change_details_json")
public func ph_change_asset_collection_change_details_json(
    _ change: UnsafeMutableRawPointer?,
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let change else {
        pkrSetMessageError(outError, message: "missing PHChange")
        return nil
    }
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return nil
    }

    do {
        let changeBox = pkrBorrow(change, as: PKRChangeBox.self)
        let collection = try pkrRequestAssetCollection(localIdentifier: String(cString: collectionIdentifier))
        let payload = changeBox.change.changeDetails(for: collection).map {
            pkrEncodeObjectChangeDetails($0, transform: pkrEncodeCollection)
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_change_collection_list_change_details_json")
public func ph_change_collection_list_change_details_json(
    _ change: UnsafeMutableRawPointer?,
    _ collectionIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let change else {
        pkrSetMessageError(outError, message: "missing PHChange")
        return nil
    }
    guard let collectionIdentifier else {
        pkrSetMessageError(outError, message: "missing collection identifier")
        return nil
    }

    do {
        let changeBox = pkrBorrow(change, as: PKRChangeBox.self)
        let collectionList = try pkrRequestCollectionList(localIdentifier: String(cString: collectionIdentifier))
        let payload = changeBox.change.changeDetails(for: collectionList).map {
            pkrEncodeObjectChangeDetails($0, transform: pkrEncodeCollectionList)
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
