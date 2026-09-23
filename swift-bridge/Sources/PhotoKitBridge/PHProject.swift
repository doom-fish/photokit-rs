import AppKit
import Foundation
import Photos

struct PKRProjectPayload: Codable {
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
    var projectExtensionDataBase64: String
    var hasProjectPreview: Bool
}

struct PKRProjectChangeRequestPayload: Codable {
    var projectLocalIdentifier: String
    var title: String?
    var projectExtensionDataBase64: String?
    var projectPreviewImageFileURL: String?
    var removeAssetIdentifiers: [String]
}

func pkrEncodeProject(_ project: PHProject) -> PKRProjectPayload {
    let estimatedAssetCount: UInt64? = project.estimatedAssetCount == NSNotFound ? nil : UInt64(exactly: project.estimatedAssetCount)
    return PKRProjectPayload(
        localIdentifier: project.localIdentifier,
        localizedTitle: project.localizedTitle,
        collectionType: pkrCollectionType(from: project.assetCollectionType),
        collectionSubtype: project.assetCollectionSubtype.rawValue,
        estimatedAssetCount: estimatedAssetCount,
        startDate: pkrDateString(project.startDate),
        endDate: pkrDateString(project.endDate),
        approximateLocation: pkrCoordinatePayload(project.approximateLocation),
        localizedLocationNames: project.localizedLocationNames,
        canContainAssets: project.canContainAssets,
        canContainCollections: project.canContainCollections,
        projectExtensionDataBase64: project.projectExtensionData.base64EncodedString(),
        hasProjectPreview: project.hasProjectPreview
    )
}

func pkrRequestProject(localIdentifier: String) throws -> PHProject {
    if let project = PHAssetCollection.fetchAssetCollections(withLocalIdentifiers: [localIdentifier], options: nil).firstObject as? PHProject {
        return project
    }
    throw NSError(
        domain: "photokit-rs",
        code: -1,
        userInfo: [NSLocalizedDescriptionKey: "project not found: \(localIdentifier)"]
    )
}

@_cdecl("ph_project_fetch_top_level_json")
public func ph_project_fetch_top_level_json(
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = try pkrCheckedFetchResult(PHCollection.fetchTopLevelUserCollections(with: pkrBuildFetchOptions(payload, for: .collection)))
        var projects: [PKRProjectPayload] = []
        result.enumerateObjects { collection, _, _ in
            if let project = collection as? PHProject {
                projects.append(pkrEncodeProject(project))
            }
        }
        return pkrCString(try pkrEncodeJSON(projects))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_project_fetch_with_local_identifiers_json")
public func ph_project_fetch_with_local_identifiers_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ fetchOptionsJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        _ = try pkrDecodeJSON(fetchOptionsJSON, as: PKRFetchOptionsPayload.self)
        let result = PHAssetCollection.fetchAssetCollections(withLocalIdentifiers: identifiers, options: nil)
        var projects: [PKRProjectPayload] = []
        result.enumerateObjects { collection, _, _ in
            if let project = collection as? PHProject {
                projects.append(pkrEncodeProject(project))
            }
        }
        return pkrCString(try pkrEncodeJSON(projects))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_project_change_request_perform_json")
public func ph_project_change_request_perform_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let payload = try pkrDecodeJSON(payloadJSON, as: PKRProjectChangeRequestPayload.self)
        let project = try pkrRequestProject(localIdentifier: payload.projectLocalIdentifier)
        let extensionData = try payload.projectExtensionDataBase64.map { dataBase64 -> Data in
            guard let data = Data(base64Encoded: dataBase64) else {
                throw pkrError("project extension data is not valid base64")
            }
            return data
        }
        let previewImage = try payload.projectPreviewImageFileURL.map { previewURL -> NSImage in
            let url = try pkrReadableFileURL(previewURL)
            guard let image = NSImage(contentsOf: url) else {
                throw pkrError("project preview image is not a decodable image: \(url.path)")
            }
            return image
        }
        let assetsToRemove = try payload.removeAssetIdentifiers.map(pkrRequestAsset)
        try PHPhotoLibrary.shared().performChangesAndWait {
            let request = PHProjectChangeRequest(project: project)
            if let title = payload.title {
                request.title = title
            }
            if let extensionData {
                request.projectExtensionData = extensionData
            }
            if let previewImage {
                request.setProjectPreviewImage(previewImage)
            }
            if !assetsToRemove.isEmpty {
                request.removeAssets(assetsToRemove)
            }
        }
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}
