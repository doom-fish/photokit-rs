import AppKit
import Foundation
import Photos
import UniformTypeIdentifiers

public typealias PKRAsyncJSONCallback = @convention(c) (
    UnsafePointer<CChar>?,
    UnsafePointer<CChar>?,
    UnsafeMutableRawPointer
) -> Void

struct PKRAuthorizationStatusPayload: Codable {
    var status: Int32
}

@_cdecl("ph_photo_library_request_authorization_async")
public func ph_photo_library_request_authorization_async(
    _ accessLevelRaw: Int32,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    func fire(status: PHAuthorizationStatus) {
        if let json = try? pkrEncodeJSON(
            PKRAuthorizationStatusPayload(status: Int32(status.rawValue))
        ) {
            json.withCString { cb($0, nil, ctx) }
        } else {
            "encode failed".withCString { cb(nil, $0, ctx) }
        }
    }

    if #available(macOS 11.0, *) {
        do {
            let level = try pkrAccessLevel(rawValue: accessLevelRaw)
            PHPhotoLibrary.requestAuthorization(for: level) { status in
                fire(status: status)
            }
        } catch {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        }
    } else {
        PHPhotoLibrary.requestAuthorization { status in
            fire(status: status)
        }
    }
}

func pkrPerformChanges(
    _ change: @escaping () throws -> String?,
    callback cb: PKRAsyncJSONCallback,
    context ctx: UnsafeMutableRawPointer
) {
    var placeholderLocalIdentifier: String?
    var changeError: Error?
    PHPhotoLibrary.shared().performChanges({
        do {
            placeholderLocalIdentifier = try change()
        } catch {
            changeError = error
        }
    }) { _, error in
        if let error = changeError ?? error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else if let json = try? pkrEncodeJSON(
            PKRChangeRequestPerformResultPayload(
                placeholderLocalIdentifier: placeholderLocalIdentifier
            )
        ) {
            json.withCString { cb($0, nil, ctx) }
        } else {
            "encode failed".withCString { cb(nil, $0, ctx) }
        }
    }
}

@_cdecl("ph_asset_change_request_perform_async")
public func ph_asset_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    let change: PKRResolvedAssetChange
    do {
        change = try pkrResolveAssetChange(
            try pkrDecodeJSON(payloadJSON, as: PKRAssetChangeRequestPayload.self)
        )
    } catch {
        error.localizedDescription.withCString { cb(nil, $0, ctx) }
        return
    }
    pkrPerformChanges({ try pkrApplyAssetChange(change) }, callback: cb, context: ctx)
}

@_cdecl("ph_asset_collection_change_request_perform_async")
public func ph_asset_collection_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    let change: PKRResolvedAssetCollectionChange
    do {
        change = try pkrResolveAssetCollectionChange(
            try pkrDecodeJSON(payloadJSON, as: PKRAssetCollectionChangeRequestPayload.self)
        )
    } catch {
        error.localizedDescription.withCString { cb(nil, $0, ctx) }
        return
    }
    pkrPerformChanges({ try pkrApplyAssetCollectionChange(change) }, callback: cb, context: ctx)
}

@_cdecl("ph_collection_list_change_request_perform_async")
public func ph_collection_list_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    let change: PKRResolvedCollectionListChange
    do {
        change = try pkrResolveCollectionListChange(
            try pkrDecodeJSON(payloadJSON, as: PKRCollectionListChangeRequestPayload.self)
        )
    } catch {
        error.localizedDescription.withCString { cb(nil, $0, ctx) }
        return
    }
    pkrPerformChanges({ try pkrApplyCollectionListChange(change) }, callback: cb, context: ctx)
}

@_cdecl("ph_image_manager_request_image_async")
public func ph_image_manager_request_image_async(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let manager,
          let assetIdentifier,
          let request = try? pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self) else {
        "invalid image request args".withCString { cb(nil, $0, ctx) }
        return
    }

    let mgr = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
    let identifier = String(cString: assetIdentifier)
    let assets = PHAsset.fetchAssets(withLocalIdentifiers: [identifier], options: nil)
    guard let asset = assets.firstObject else {
        "asset not found: \(identifier)".withCString { cb(nil, $0, ctx) }
        return
    }

    let options = pkrBuildImageRequestOptions(request)
    let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
    let contentMode = pkrContentMode(from: request.contentMode)
    var fired = false
    mgr.requestImage(
        for: asset,
        targetSize: targetSize,
        contentMode: contentMode,
        options: options
    ) { image, info in
        let isDegraded = (info?[PHImageResultIsDegradedKey] as? Bool) ?? false
        let isCancelled = (info?[PHImageCancelledKey] as? Bool) ?? false
        guard !fired, (!isDegraded || isCancelled) else { return }
        fired = true

        let requestID = info.flatMap { pkrRequestID(from: $0) }
        let errorPayload = info.flatMap { pkrResultErrorPayload(from: $0, key: PHImageErrorKey) }

        if isCancelled {
            let payload = PKRImageResultPayload(
                tiffDataBase64: "",
                width: 0,
                height: 0,
                cancelled: true,
                degraded: false,
                requestID: requestID,
                error: nil
            )
            if let json = try? pkrEncodeJSON(payload) {
                json.withCString { cb($0, nil, ctx) }
            } else {
                "encode failed".withCString { cb(nil, $0, ctx) }
            }
            return
        }

        guard let nsImage = image, let tiffData = nsImage.tiffRepresentation else {
            if let errPayload = errorPayload {
                let payload = PKRImageResultPayload(
                    tiffDataBase64: "",
                    width: 0,
                    height: 0,
                    cancelled: false,
                    degraded: false,
                    requestID: requestID,
                    error: errPayload
                )
                if let json = try? pkrEncodeJSON(payload) {
                    json.withCString { cb($0, nil, ctx) }
                } else {
                    "image request failed".withCString { cb(nil, $0, ctx) }
                }
            } else {
                "image not available".withCString { cb(nil, $0, ctx) }
            }
            return
        }

        let payload = PKRImageResultPayload(
            tiffDataBase64: tiffData.base64EncodedString(),
            width: nsImage.size.width,
            height: nsImage.size.height,
            cancelled: false,
            degraded: false,
            requestID: requestID,
            error: errorPayload
        )
        if let json = try? pkrEncodeJSON(payload) {
            json.withCString { cb($0, nil, ctx) }
        } else {
            "encode failed".withCString { cb(nil, $0, ctx) }
        }
    }
}

@_cdecl("ph_image_manager_request_image_data_async")
public func ph_image_manager_request_image_data_async(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let manager,
          let assetIdentifier,
          let request = try? pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self) else {
        "invalid image data request args".withCString { cb(nil, $0, ctx) }
        return
    }

    let mgr = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
    let identifier = String(cString: assetIdentifier)
    let assets = PHAsset.fetchAssets(withLocalIdentifiers: [identifier], options: nil)
    guard let asset = assets.firstObject else {
        "asset not found: \(identifier)".withCString { cb(nil, $0, ctx) }
        return
    }

    let options = pkrBuildImageRequestOptions(request)
    var fired = false
    mgr.requestImageDataAndOrientation(for: asset, options: options) { data, uti, orientation, info in
        let isDegraded = (info?[PHImageResultIsDegradedKey] as? Bool) ?? false
        let isCancelled = (info?[PHImageCancelledKey] as? Bool) ?? false
        guard !fired, (!isDegraded || isCancelled) else { return }
        fired = true

        let requestID = info.flatMap { pkrRequestID(from: $0) }
        let isInCloud = (info?[PHImageResultIsInCloudKey] as? Bool) ?? false
        let errorPayload = info.flatMap { pkrResultErrorPayload(from: $0, key: PHImageErrorKey) }
        let contentTypeID: String?
        if #available(macOS 11.0, *) {
            contentTypeID = uti.flatMap { UTType($0)?.identifier }
        } else {
            contentTypeID = nil
        }

        let payload = PKRImageDataResultPayload(
            dataBase64: data?.base64EncodedString() ?? "",
            uniformTypeIdentifier: uti,
            contentTypeIdentifier: contentTypeID,
            orientation: Int32(orientation.rawValue),
            cancelled: isCancelled,
            degraded: isDegraded,
            isInCloud: isInCloud,
            requestID: requestID,
            error: errorPayload
        )
        if let json = try? pkrEncodeJSON(payload) {
            json.withCString { cb($0, nil, ctx) }
        } else {
            "encode failed".withCString { cb(nil, $0, ctx) }
        }
    }
}

@_cdecl("ph_live_photo_editing_context_save_async")
public func ph_live_photo_editing_context_save_async(
    _ context: UnsafeMutableRawPointer?,
    _ output: UnsafeMutableRawPointer?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let context, let output else {
        "missing context or output".withCString { cb(nil, $0, ctx) }
        return
    }

    let contextBox = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self)
    let outputBox = pkrBorrow(output, as: PKRContentEditingOutputBox.self)
    contextBox.context.saveLivePhoto(to: outputBox.output, options: nil) { success, error in
        withExtendedLifetime((contextBox, outputBox)) {
            if let error {
                error.localizedDescription.withCString { cb(nil, $0, ctx) }
                return
            }

            let payload = PKRLivePhotoEditingSaveResultPayload(success: success)
            if let json = try? pkrEncodeJSON(payload) {
                json.withCString { cb($0, nil, ctx) }
            } else {
                "encode failed".withCString { cb(nil, $0, ctx) }
            }
        }
    }
}

@_cdecl("ph_live_photo_editing_context_prepare_async")
public func ph_live_photo_editing_context_prepare_async(
    _ context: UnsafeMutableRawPointer?,
    _ targetWidth: Double,
    _ targetHeight: Double,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let context else {
        "missing context".withCString { cb(nil, $0, ctx) }
        return
    }

    let contextBox = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self)
    contextBox.context.prepareLivePhotoForPlayback(
        withTargetSize: CGSize(width: targetWidth, height: targetHeight),
        options: nil
    ) { livePhoto, error in
        withExtendedLifetime(contextBox) {
            if let error {
                error.localizedDescription.withCString { cb(nil, $0, ctx) }
                return
            }

            let payload = PKRLivePhotoResultPayload(
                hasLivePhoto: livePhoto != nil,
                cancelled: false,
                degraded: false,
                sizeWidth: Double(livePhoto?.size.width ?? 0),
                sizeHeight: Double(livePhoto?.size.height ?? 0),
                requestID: nil,
                error: nil
            )
            if let json = try? pkrEncodeJSON(payload) {
                json.withCString { cb($0, nil, ctx) }
            } else {
                "encode failed".withCString { cb(nil, $0, ctx) }
            }
        }
    }
}
