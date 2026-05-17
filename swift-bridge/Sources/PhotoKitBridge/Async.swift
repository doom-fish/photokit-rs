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

@_cdecl("ph_asset_change_request_perform_async")
public func ph_asset_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let payload = try? pkrDecodeJSON(payloadJSON, as: PKRAssetChangeRequestPayload.self) else {
        "invalid asset change request payload".withCString { cb(nil, $0, ctx) }
        return
    }

    var placeholderLocalIdentifier: String? = nil
    PHPhotoLibrary.shared().performChanges({
        let request: PHAssetChangeRequest
        if let assetLocalIdentifier = payload.assetLocalIdentifier {
            request = PHAssetChangeRequest(for: try! pkrRequestAsset(localIdentifier: assetLocalIdentifier))
        } else if let imageFileURL = payload.createImageFileURL {
            guard let created = PHAssetChangeRequest.creationRequestForAssetFromImage(
                atFileURL: pkrAssetCreationURL(imageFileURL)
            ) else {
                return
            }
            request = created
        } else if let imageDataBase64 = payload.createImageDataBase64,
                  let data = Data(base64Encoded: imageDataBase64),
                  let image = NSImage(data: data) {
            request = PHAssetChangeRequest.creationRequestForAsset(from: image)
        } else if let videoFileURL = payload.createVideoFileURL,
                  let created = PHAssetChangeRequest.creationRequestForAssetFromVideo(
                    atFileURL: pkrAssetCreationURL(videoFileURL)
                  ) {
            request = created
        } else {
            return
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
    }) { _, error in
        if let error {
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

@_cdecl("ph_asset_collection_change_request_perform_async")
public func ph_asset_collection_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let payload = try? pkrDecodeJSON(
        payloadJSON,
        as: PKRAssetCollectionChangeRequestPayload.self
    ) else {
        "invalid collection change request payload".withCString { cb(nil, $0, ctx) }
        return
    }

    var placeholderLocalIdentifier: String? = nil
    PHPhotoLibrary.shared().performChanges({
        let request: PHAssetCollectionChangeRequest
        if let creationTitle = payload.creationTitle {
            request = PHAssetCollectionChangeRequest.creationRequestForAssetCollection(
                withTitle: creationTitle
            )
        } else if let identifier = payload.assetCollectionLocalIdentifier,
                  let existing = PHAssetCollectionChangeRequest(
                    for: try! pkrRequestAssetCollection(localIdentifier: identifier)
                  ) {
            request = existing
        } else {
            return
        }

        if let title = payload.title {
            request.title = title
        }
        for mutation in payload.assetMutations {
            try! pkrApplyAssetCollectionMutation(mutation, to: request)
        }
        placeholderLocalIdentifier = request.placeholderForCreatedAssetCollection.localIdentifier
    }) { _, error in
        if let error {
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

@_cdecl("ph_collection_list_change_request_perform_async")
public func ph_collection_list_change_request_perform_async(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ cb: PKRAsyncJSONCallback,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let payload = try? pkrDecodeJSON(
        payloadJSON,
        as: PKRCollectionListChangeRequestPayload.self
    ) else {
        "invalid collection list change request payload".withCString { cb(nil, $0, ctx) }
        return
    }

    var placeholderLocalIdentifier: String? = nil
    PHPhotoLibrary.shared().performChanges({
        let request: PHCollectionListChangeRequest
        if let creationTitle = payload.creationTitle {
            request = PHCollectionListChangeRequest.creationRequestForCollectionList(
                withTitle: creationTitle
            )
        } else if payload.topLevelUserCollections {
            let result = PHCollection.fetchTopLevelUserCollections(with: nil)
            guard let topLevel = PHCollectionListChangeRequest(
                forTopLevelCollectionListUserCollections: result
            ) else {
                return
            }
            request = topLevel
        } else if let identifier = payload.collectionListLocalIdentifier,
                  let existing = PHCollectionListChangeRequest(
                    for: try! pkrRequestCollectionList(localIdentifier: identifier)
                  ) {
            request = existing
        } else {
            return
        }

        if let title = payload.title {
            request.title = title
        }
        for mutation in payload.childMutations {
            try! pkrApplyCollectionListMutation(mutation, to: request)
        }
        placeholderLocalIdentifier = request.placeholderForCreatedCollectionList.localIdentifier
    }) { _, error in
        if let error {
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

    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    let editingOutput = pkrBorrow(output, as: PHContentEditingOutput.self)
    editingContext.saveLivePhoto(to: editingOutput, options: nil) { success, error in
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

    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    editingContext.prepareLivePhotoForPlayback(
        withTargetSize: CGSize(width: targetWidth, height: targetHeight),
        options: nil
    ) { livePhoto, error in
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
