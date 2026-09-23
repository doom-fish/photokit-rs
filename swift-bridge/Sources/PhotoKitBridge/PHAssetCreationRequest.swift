import Foundation
import Photos
import UniformTypeIdentifiers

struct PKRAssetResourceCreationOptionsPayload: Codable {
    var originalFilename: String?
    var contentTypeIdentifier: String?
    var uniformTypeIdentifier: String?
    var shouldMoveFile: Bool
}

struct PKRAssetCreationResourcePayload: Codable {
    var resourceType: Int
    var fileURL: String?
    var dataBase64: String?
    var options: PKRAssetResourceCreationOptionsPayload?
}

func pkrAssetResourceCreationOptions(
    from payload: PKRAssetResourceCreationOptionsPayload?
) -> PHAssetResourceCreationOptions? {
    guard let payload else { return nil }
    let options = PHAssetResourceCreationOptions()
    options.originalFilename = payload.originalFilename
    options.shouldMoveFile = payload.shouldMoveFile
    if #available(macOS 26.0, *) {
        if let contentTypeIdentifier = payload.contentTypeIdentifier,
           let contentType = UTType(contentTypeIdentifier) {
            options.contentType = contentType
        }
    }
    if let uniformTypeIdentifier = payload.uniformTypeIdentifier {
        options.uniformTypeIdentifier = uniformTypeIdentifier
    }
    return options
}

enum PKRAssetCreationSource {
    case file(URL)
    case data(Data)
}

struct PKRResolvedAssetCreationResource {
    var type: PHAssetResourceType
    var source: PKRAssetCreationSource
    var options: PHAssetResourceCreationOptions?
}

func pkrResolveAssetCreationResource(_ resource: PKRAssetCreationResourcePayload) throws -> PKRResolvedAssetCreationResource {
    guard let type = PHAssetResourceType(rawValue: resource.resourceType) else {
        throw pkrError("unknown asset resource type: \(resource.resourceType)")
    }
    let source: PKRAssetCreationSource
    if let fileURL = resource.fileURL {
        source = .file(try pkrReadableFileURL(fileURL))
    } else if let dataBase64 = resource.dataBase64 {
        guard let data = Data(base64Encoded: dataBase64) else {
            throw pkrError("asset resource data is not valid base64")
        }
        source = .data(data)
    } else {
        throw pkrError("asset resource needs a file URL or data")
    }
    return PKRResolvedAssetCreationResource(
        type: type,
        source: source,
        options: pkrAssetResourceCreationOptions(from: resource.options)
    )
}

@_cdecl("ph_asset_creation_request_supports_resource_types")
public func ph_asset_creation_request_supports_resource_types(
    _ resourceTypesJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let resourceTypes = try pkrDecodeJSON(resourceTypesJSON, as: [Int].self)
        return PHAssetCreationRequest.supportsAssetResourceTypes(resourceTypes.map(NSNumber.init(value:))) ? 1 : 0
    } catch {
        pkrSetError(outError, error)
        return 0
    }
}

@_cdecl("ph_asset_creation_request_perform")
public func ph_asset_creation_request_perform(
    _ resourcesJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let resources = try pkrDecodeJSON(resourcesJSON, as: [PKRAssetCreationResourcePayload].self)
        guard !resources.isEmpty else {
            throw NSError(
                domain: "photokit-rs",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: "asset creation requires at least one resource"]
            )
        }

        guard PHAssetCreationRequest.supportsAssetResourceTypes(resources.map { NSNumber(value: $0.resourceType) }) else {
            throw pkrError("unsupported combination of asset resource types: \(resources.map(\.resourceType))")
        }
        let resolved = try resources.map(pkrResolveAssetCreationResource)

        var localIdentifier: String?
        try PHPhotoLibrary.shared().performChangesAndWait {
            let request = PHAssetCreationRequest.forAsset()
            for resource in resolved {
                switch resource.source {
                case .file(let url):
                    request.addResource(with: resource.type, fileURL: url, options: resource.options)
                case .data(let data):
                    request.addResource(with: resource.type, data: data, options: resource.options)
                }
            }
            localIdentifier = request.placeholderForCreatedAsset?.localIdentifier
        }

        guard let localIdentifier else {
            throw NSError(
                domain: "photokit-rs",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: "missing placeholder local identifier after asset creation"]
            )
        }
        return pkrCString(localIdentifier)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
