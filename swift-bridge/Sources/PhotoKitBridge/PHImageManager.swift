import AppKit
import AVFoundation
import Foundation
import Photos

final class PKRImageManagerBox: NSObject {
    let manager: PHImageManager

    init(manager: PHImageManager) {
        self.manager = manager
        super.init()
    }
}

final class PKRCachingImageManagerBox: NSObject {
    let manager: PHCachingImageManager

    init(manager: PHCachingImageManager) {
        self.manager = manager
        super.init()
    }
}

final class PKRRequestBox: NSObject {
    var requestID: Int32 = 0
    var payloadJSON: String?
    var cancelPayloadJSON: String?
    var error: Error?
    var completed = false
    var cancelHandler: (() -> Void)?
    let semaphore = DispatchSemaphore(value: 0)

    func finish<T: Encodable>(_ payload: T) {
        guard !completed else { return }
        payloadJSON = try? pkrEncodeJSON(payload)
        completed = true
        semaphore.signal()
    }

    func fail(_ error: Error) {
        guard !completed else { return }
        self.error = error
        completed = true
        semaphore.signal()
    }

    func cancel() {
        cancelHandler?()
        guard !completed else { return }
        payloadJSON = cancelPayloadJSON
        completed = true
        semaphore.signal()
    }
}

enum PKRImageContentMode: String, Codable {
    case `default`
    case aspectFit
    case aspectFill
}

struct PKRImageRequestPayload: Codable {
    var targetWidth: Double
    var targetHeight: Double
    var contentMode: PKRImageContentMode
    var version: String?
    var deliveryMode: String?
    var resizeMode: String?
    var networkAccessAllowed: Bool
    var synchronous: Bool
    var allowSecondaryDegradedImage: Bool
}

struct PKRImageResultPayload: Codable {
    var tiffDataBase64: String
    var width: Double
    var height: Double
    var cancelled: Bool
    var degraded: Bool
    var requestID: Int32?
    var error: PKRErrorPayload?
}

struct PKRImageDataResultPayload: Codable {
    var dataBase64: String
    var uniformTypeIdentifier: String?
    var contentTypeIdentifier: String?
    var orientation: Int32
    var cancelled: Bool
    var degraded: Bool
    var isInCloud: Bool
    var requestID: Int32?
    var error: PKRErrorPayload?
}

struct PKRLivePhotoResultPayload: Codable {
    var hasLivePhoto: Bool
    var cancelled: Bool
    var degraded: Bool
    var sizeWidth: Double
    var sizeHeight: Double
    var requestID: Int32?
    var error: PKRErrorPayload?
}

struct PKRVideoRequestOptionsPayload: Codable {
    var networkAccessAllowed: Bool
    var version: Int
    var deliveryMode: Int
}

struct PKRVideoResultPayload: Codable {
    var resultType: String
    var requestID: Int32?
    var cancelled: Bool
    var isInCloud: Bool
    var error: PKRErrorPayload?
    var assetURL: String?
    var durationSeconds: Double?
    var hasPlayerItem: Bool
    var hasExportSession: Bool
    var hasAVAsset: Bool
    var hasAudioMix: Bool
    var exportPreset: String?
    var supportedFileTypes: [String]
}

func pkrContentMode(from contentMode: PKRImageContentMode) -> PHImageContentMode {
    switch contentMode {
    case .default:
        return .default
    case .aspectFit:
        return .aspectFit
    case .aspectFill:
        return .aspectFill
    }
}

func pkrRequestID(from info: [AnyHashable: Any]) -> Int32? {
    (info[PHImageResultRequestIDKey] as? NSNumber).map { Int32(truncating: $0) }
}

func pkrResultErrorPayload(from info: [AnyHashable: Any], key: String) -> PKRErrorPayload? {
    (info[key] as? NSError).map(pkrErrorPayload)
}

func pkrBuildImageRequestOptions(_ payload: PKRImageRequestPayload) -> PHImageRequestOptions {
    let options = PHImageRequestOptions()
    if let version = payload.version {
        switch version {
        case "current":
            options.version = .current
        case "unadjusted":
            options.version = .unadjusted
        case "original":
            options.version = .original
        default:
            break
        }
    }
    if let deliveryMode = payload.deliveryMode {
        switch deliveryMode {
        case "opportunistic":
            options.deliveryMode = .opportunistic
        case "highQualityFormat":
            options.deliveryMode = .highQualityFormat
        case "fastFormat":
            options.deliveryMode = .fastFormat
        default:
            break
        }
    }
    if let resizeMode = payload.resizeMode {
        switch resizeMode {
        case "none":
            options.resizeMode = .none
        case "fast":
            options.resizeMode = .fast
        case "exact":
            options.resizeMode = .exact
        default:
            break
        }
    }
    options.isNetworkAccessAllowed = payload.networkAccessAllowed
    options.isSynchronous = payload.synchronous
    if #available(macOS 14.0, *) {
        options.allowSecondaryDegradedImage = payload.allowSecondaryDegradedImage
    }
    return options
}

func pkrBuildLivePhotoRequestOptions(_ payload: PKRImageRequestPayload) -> PHLivePhotoRequestOptions {
    let options = PHLivePhotoRequestOptions()
    if let version = payload.version {
        switch version {
        case "current":
            options.version = .current
        case "unadjusted":
            options.version = .unadjusted
        case "original":
            options.version = .original
        default:
            break
        }
    }
    if let deliveryMode = payload.deliveryMode {
        switch deliveryMode {
        case "opportunistic":
            options.deliveryMode = .opportunistic
        case "highQualityFormat":
            options.deliveryMode = .highQualityFormat
        case "fastFormat":
            options.deliveryMode = .fastFormat
        default:
            break
        }
    }
    options.isNetworkAccessAllowed = payload.networkAccessAllowed
    return options
}

func pkrBuildVideoRequestOptions(_ payload: PKRVideoRequestOptionsPayload) -> PHVideoRequestOptions {
    let options = PHVideoRequestOptions()
    options.isNetworkAccessAllowed = payload.networkAccessAllowed
    options.version = PHVideoRequestOptionsVersion(rawValue: payload.version) ?? .current
    options.deliveryMode = PHVideoRequestOptionsDeliveryMode(rawValue: payload.deliveryMode) ?? .automatic
    return options
}

func pkrLivePhotoResultPayload(_ livePhoto: PHLivePhoto?, info: [AnyHashable: Any]) -> PKRLivePhotoResultPayload {
    PKRLivePhotoResultPayload(
        hasLivePhoto: livePhoto != nil,
        cancelled: (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false,
        degraded: (info[PHImageResultIsDegradedKey] as? NSNumber)?.boolValue ?? false,
        sizeWidth: livePhoto.map { Double($0.size.width) } ?? 0,
        sizeHeight: livePhoto.map { Double($0.size.height) } ?? 0,
        requestID: pkrRequestID(from: info),
        error: pkrResultErrorPayload(from: info, key: PHLivePhotoInfoErrorKey)
    )
}

func pkrAssetURLAndDuration(from asset: AVAsset?) -> (String?, Double?) {
    guard let asset else { return (nil, nil) }
    let url = (asset as? AVURLAsset)?.url.absoluteString
    let duration = asset.duration.isNumeric ? CMTimeGetSeconds(asset.duration) : nil
    return (url, duration)
}

func pkrVideoResultPayload(
    resultType: String,
    requestID: Int32,
    info: [AnyHashable: Any],
    asset: AVAsset?,
    hasPlayerItem: Bool,
    hasExportSession: Bool,
    hasAVAsset: Bool,
    hasAudioMix: Bool,
    exportPreset: String?,
    supportedFileTypes: [String]
) -> PKRVideoResultPayload {
    let (assetURL, durationSeconds) = pkrAssetURLAndDuration(from: asset)
    return PKRVideoResultPayload(
        resultType: resultType,
        requestID: requestID,
        cancelled: (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false,
        isInCloud: (info[PHImageResultIsInCloudKey] as? NSNumber)?.boolValue ?? false,
        error: pkrResultErrorPayload(from: info, key: PHImageErrorKey),
        assetURL: assetURL,
        durationSeconds: durationSeconds,
        hasPlayerItem: hasPlayerItem,
        hasExportSession: hasExportSession,
        hasAVAsset: hasAVAsset,
        hasAudioMix: hasAudioMix,
        exportPreset: exportPreset,
        supportedFileTypes: supportedFileTypes
    )
}

func pkrWaitForRequestBox<T: Encodable>(
    timeoutMs: UInt64,
    cancel: (() -> Void)?,
    work: (_ finish: @escaping (T) -> Void, _ fail: @escaping (Error) -> Void) -> Void
) throws -> T {
    let semaphore = DispatchSemaphore(value: 0)
    var payload: T?
    var requestError: Error?
    work({ result in
        payload = result
        semaphore.signal()
    }, { error in
        requestError = error
        semaphore.signal()
    })
    let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
    guard semaphore.wait(timeout: timeout) == .success else {
        cancel?()
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "request timed out"]
        )
    }
    if let requestError {
        throw requestError
    }
    guard let payload else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "request completed without payload"]
        )
    }
    return payload
}

@_cdecl("ph_image_manager_default")
public func ph_image_manager_default() -> UnsafeMutableRawPointer {
    pkrRetain(PKRImageManagerBox(manager: PHImageManager.default()))
}

@_cdecl("ph_caching_image_manager_new")
public func ph_caching_image_manager_new() -> UnsafeMutableRawPointer {
    pkrRetain(PKRCachingImageManagerBox(manager: PHCachingImageManager()))
}

@_cdecl("ph_image_manager_release")
public func ph_image_manager_release(_ manager: UnsafeMutableRawPointer?) {
    guard let manager else { return }
    pkrRelease(manager)
}

@_cdecl("ph_image_manager_request_image")
public func ph_image_manager_request_image(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        let box = PKRRequestBox()
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        box.cancelPayloadJSON = try? pkrEncodeJSON(
            PKRImageResultPayload(
                tiffDataBase64: "",
                width: 0,
                height: 0,
                cancelled: true,
                degraded: false,
                requestID: nil,
                error: nil
            )
        )
        box.requestID = imageManager.requestImage(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildImageRequestOptions(request)
        ) { image, info in
            let info = info ?? [:]
            let cancelled = (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false
            let degraded = (info[PHImageResultIsDegradedKey] as? NSNumber)?.boolValue ?? false
            guard let image, let data = image.tiffRepresentation else {
                box.finish(
                    PKRImageResultPayload(
                        tiffDataBase64: "",
                        width: 0,
                        height: 0,
                        cancelled: cancelled,
                        degraded: degraded,
                        requestID: pkrRequestID(from: info),
                        error: pkrResultErrorPayload(from: info, key: PHImageErrorKey)
                    )
                )
                return
            }
            box.finish(
                PKRImageResultPayload(
                    tiffDataBase64: data.base64EncodedString(),
                    width: image.size.width,
                    height: image.size.height,
                    cancelled: cancelled,
                    degraded: degraded,
                    requestID: pkrRequestID(from: info),
                    error: pkrResultErrorPayload(from: info, key: PHImageErrorKey)
                )
            )
        }
        box.cancelHandler = { imageManager.cancelImageRequest(PHImageRequestID(box.requestID)) }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_image_data")
public func ph_image_manager_request_image_data(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        let box = PKRRequestBox()
        box.cancelPayloadJSON = try? pkrEncodeJSON(
            PKRImageDataResultPayload(
                dataBase64: "",
                uniformTypeIdentifier: nil,
                contentTypeIdentifier: nil,
                orientation: 0,
                cancelled: true,
                degraded: false,
                isInCloud: false,
                requestID: nil,
                error: nil
            )
        )
        box.requestID = imageManager.requestImageDataAndOrientation(
            for: asset,
            options: pkrBuildImageRequestOptions(request)
        ) { imageData, dataUTI, orientation, info in
            let info = info ?? [:]
            box.finish(
                PKRImageDataResultPayload(
                    dataBase64: imageData?.base64EncodedString() ?? "",
                    uniformTypeIdentifier: dataUTI,
                    contentTypeIdentifier: nil,
                    orientation: Int32(orientation.rawValue),
                    cancelled: (info[PHImageCancelledKey] as? NSNumber)?.boolValue ?? false,
                    degraded: (info[PHImageResultIsDegradedKey] as? NSNumber)?.boolValue ?? false,
                    isInCloud: (info[PHImageResultIsInCloudKey] as? NSNumber)?.boolValue ?? false,
                    requestID: pkrRequestID(from: info),
                    error: pkrResultErrorPayload(from: info, key: PHImageErrorKey)
                )
            )
        }
        box.cancelHandler = { imageManager.cancelImageRequest(PHImageRequestID(box.requestID)) }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_live_photo")
public func ph_image_manager_request_live_photo(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        let box = PKRRequestBox()
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        box.cancelPayloadJSON = try? pkrEncodeJSON(
            PKRLivePhotoResultPayload(
                hasLivePhoto: false,
                cancelled: true,
                degraded: false,
                sizeWidth: 0,
                sizeHeight: 0,
                requestID: nil,
                error: nil
            )
        )
        box.requestID = imageManager.requestLivePhoto(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildLivePhotoRequestOptions(request)
        ) { livePhoto, info in
            box.finish(pkrLivePhotoResultPayload(livePhoto, info: info ?? [:]))
        }
        box.cancelHandler = { imageManager.cancelImageRequest(PHImageRequestID(box.requestID)) }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_player_item_for_video_json")
public func ph_image_manager_request_player_item_for_video_json(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let options = try pkrDecodeJSON(optionsJSON, as: PKRVideoRequestOptionsPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        var requestID: Int32 = 0
        let payload = try pkrWaitForRequestBox(timeoutMs: timeoutMs, cancel: {
            imageManager.cancelImageRequest(PHImageRequestID(requestID))
        }) { finish, _ in
            requestID = imageManager.requestPlayerItem(forVideo: asset, options: pkrBuildVideoRequestOptions(options)) { playerItem, info in
                let info = info ?? [:]
                finish(
                    pkrVideoResultPayload(
                        resultType: "playerItem",
                        requestID: requestID,
                        info: info,
                        asset: playerItem?.asset,
                        hasPlayerItem: playerItem != nil,
                        hasExportSession: false,
                        hasAVAsset: false,
                        hasAudioMix: false,
                        exportPreset: nil,
                        supportedFileTypes: []
                    )
                )
            }
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_export_session_for_video_json")
public func ph_image_manager_request_export_session_for_video_json(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ exportPreset: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }
    guard let exportPreset else {
        pkrSetMessageError(outError, message: "missing export preset")
        return nil
    }

    do {
        let options = try pkrDecodeJSON(optionsJSON, as: PKRVideoRequestOptionsPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let preset = String(cString: exportPreset)
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        var requestID: Int32 = 0
        let payload = try pkrWaitForRequestBox(timeoutMs: timeoutMs, cancel: {
            imageManager.cancelImageRequest(PHImageRequestID(requestID))
        }) { finish, _ in
            requestID = imageManager.requestExportSession(forVideo: asset, options: pkrBuildVideoRequestOptions(options), exportPreset: preset) { exportSession, info in
                let info = info ?? [:]
                finish(
                    pkrVideoResultPayload(
                        resultType: "exportSession",
                        requestID: requestID,
                        info: info,
                        asset: exportSession?.asset,
                        hasPlayerItem: false,
                        hasExportSession: exportSession != nil,
                        hasAVAsset: false,
                        hasAudioMix: false,
                        exportPreset: exportSession?.presetName,
                        supportedFileTypes: exportSession?.supportedFileTypes.map(\.rawValue) ?? []
                    )
                )
            }
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_manager_request_av_asset_for_video_json")
public func ph_image_manager_request_av_asset_for_video_json(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHImageManager")
        return nil
    }
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let options = try pkrDecodeJSON(optionsJSON, as: PKRVideoRequestOptionsPayload.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let imageManager = pkrBorrow(manager, as: PKRImageManagerBox.self).manager
        var requestID: Int32 = 0
        let payload = try pkrWaitForRequestBox(timeoutMs: timeoutMs, cancel: {
            imageManager.cancelImageRequest(PHImageRequestID(requestID))
        }) { finish, _ in
            requestID = imageManager.requestAVAsset(forVideo: asset, options: pkrBuildVideoRequestOptions(options)) { avAsset, audioMix, info in
                let info = info ?? [:]
                finish(
                    pkrVideoResultPayload(
                        resultType: "avAsset",
                        requestID: requestID,
                        info: info,
                        asset: avAsset,
                        hasPlayerItem: false,
                        hasExportSession: false,
                        hasAVAsset: avAsset != nil,
                        hasAudioMix: audioMix != nil,
                        exportPreset: nil,
                        supportedFileTypes: []
                    )
                )
            }
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_image_request_wait_json")
public func ph_image_request_wait_json(
    _ request: UnsafeMutableRawPointer?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let request else {
        pkrSetMessageError(outError, message: "missing request handle")
        return nil
    }

    let box = pkrBorrow(request, as: PKRRequestBox.self)
    if !box.completed {
        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        if box.semaphore.wait(timeout: timeout) == .timedOut {
            box.cancelHandler?()
            pkrSetMessageError(outError, message: "request timed out")
            return nil
        }
    }

    if let error = box.error {
        pkrSetError(outError, error)
        return nil
    }
    return box.payloadJSON.flatMap(pkrCString)
}

@_cdecl("ph_image_request_cancel")
public func ph_image_request_cancel(_ request: UnsafeMutableRawPointer?) {
    guard let request else { return }
    let box = pkrBorrow(request, as: PKRRequestBox.self)
    box.cancel()
}

@_cdecl("ph_image_request_release")
public func ph_image_request_release(_ request: UnsafeMutableRawPointer?) {
    guard let request else { return }
    pkrRelease(request)
}

@_cdecl("ph_caching_image_manager_start_caching")
public func ph_caching_image_manager_start_caching(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifiersJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHCachingImageManager")
        return PKR_ERROR
    }

    do {
        let identifiers = try pkrDecodeJSON(assetIdentifiersJSON, as: [String].self)
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let cachingManager = pkrBorrow(manager, as: PKRCachingImageManagerBox.self).manager
        let assets = try identifiers.map(pkrRequestAsset)
        cachingManager.startCachingImages(
            for: assets,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildImageRequestOptions(request)
        )
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_caching_image_manager_stop_caching")
public func ph_caching_image_manager_stop_caching(
    _ manager: UnsafeMutableRawPointer?,
    _ assetIdentifiersJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let manager else {
        pkrSetMessageError(outError, message: "missing PHCachingImageManager")
        return PKR_ERROR
    }

    do {
        let identifiers = try pkrDecodeJSON(assetIdentifiersJSON, as: [String].self)
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let cachingManager = pkrBorrow(manager, as: PKRCachingImageManagerBox.self).manager
        let assets = try identifiers.map(pkrRequestAsset)
        cachingManager.stopCachingImages(
            for: assets,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildImageRequestOptions(request)
        )
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_caching_image_manager_stop_caching_all")
public func ph_caching_image_manager_stop_caching_all(_ manager: UnsafeMutableRawPointer?) {
    guard let manager else { return }
    let cachingManager = pkrBorrow(manager, as: PKRCachingImageManagerBox.self).manager
    cachingManager.stopCachingImagesForAllAssets()
}
