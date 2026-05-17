import Foundation
import Photos

struct PKRAssetResourceRequestOptionsPayload: Codable {
    var networkAccessAllowed: Bool
}

struct PKRAssetResourceDataResultPayload: Codable {
    var requestID: Int32
    var dataBase64: String
    var error: PKRErrorPayload?
}

struct PKRAssetResourceWriteResultPayload: Codable {
    var fileURL: String
    var success: Bool
    var error: PKRErrorPayload?
}

func pkrRequestAssetResource(from payload: PKRAssetResourcePayload) throws -> PHAssetResource {
    let asset = try pkrRequestAsset(localIdentifier: payload.assetLocalIdentifier)
    guard let resource = PHAssetResource.assetResources(for: asset).first(where: {
        $0.type.rawValue == payload.resourceType && $0.originalFilename == payload.originalFilename
    }) else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "asset resource not found: \(payload.originalFilename)"]
        )
    }
    return resource
}

func pkrBuildAssetResourceRequestOptions(_ payload: PKRAssetResourceRequestOptionsPayload) -> PHAssetResourceRequestOptions {
    let options = PHAssetResourceRequestOptions()
    options.isNetworkAccessAllowed = payload.networkAccessAllowed
    return options
}

func pkrAssetResourceFileURL(_ value: String) -> URL {
    if value.hasPrefix("file://") {
        return URL(string: value) ?? URL(fileURLWithPath: value)
    }
    return URL(fileURLWithPath: value)
}

@_cdecl("ph_asset_resource_manager_request_data_json")
public func ph_asset_resource_manager_request_data_json(
    _ resourceJSON: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let resourcePayload = try pkrDecodeJSON(resourceJSON, as: PKRAssetResourcePayload.self)
        let optionsPayload = try pkrDecodeJSON(optionsJSON, as: PKRAssetResourceRequestOptionsPayload.self)
        let resource = try pkrRequestAssetResource(from: resourcePayload)
        let manager = PHAssetResourceManager.default()
        let semaphore = DispatchSemaphore(value: 0)
        var received = Data()
        var requestError: NSError?
        let requestID = manager.requestData(for: resource, options: pkrBuildAssetResourceRequestOptions(optionsPayload)) { data in
            received.append(data)
        } completionHandler: { error in
            requestError = error as NSError?
            semaphore.signal()
        }

        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        guard semaphore.wait(timeout: timeout) == .success else {
            manager.cancelDataRequest(requestID)
            throw NSError(
                domain: "photokit-rs",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: "asset resource request timed out"]
            )
        }

        let payload = PKRAssetResourceDataResultPayload(
            requestID: requestID,
            dataBase64: received.base64EncodedString(),
            error: requestError.map(pkrErrorPayload)
        )
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_resource_manager_write_data_json")
public func ph_asset_resource_manager_write_data_json(
    _ resourceJSON: UnsafePointer<CChar>?,
    _ fileURL: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let fileURL else {
        pkrSetMessageError(outError, message: "missing file URL")
        return nil
    }

    do {
        let resourcePayload = try pkrDecodeJSON(resourceJSON, as: PKRAssetResourcePayload.self)
        let optionsPayload = try pkrDecodeJSON(optionsJSON, as: PKRAssetResourceRequestOptionsPayload.self)
        let resource = try pkrRequestAssetResource(from: resourcePayload)
        let manager = PHAssetResourceManager.default()
        let destinationURL = pkrAssetResourceFileURL(String(cString: fileURL))
        let semaphore = DispatchSemaphore(value: 0)
        var requestError: NSError?
        manager.writeData(for: resource, toFile: destinationURL, options: pkrBuildAssetResourceRequestOptions(optionsPayload)) { error in
            requestError = error as NSError?
            semaphore.signal()
        }

        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        guard semaphore.wait(timeout: timeout) == .success else {
            throw NSError(
                domain: "photokit-rs",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: "asset resource write timed out"]
            )
        }

        let payload = PKRAssetResourceWriteResultPayload(
            fileURL: destinationURL.absoluteString,
            success: requestError == nil,
            error: requestError.map(pkrErrorPayload)
        )
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
