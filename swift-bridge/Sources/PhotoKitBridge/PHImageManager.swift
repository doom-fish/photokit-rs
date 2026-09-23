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
    private let lock = NSLock()
    private let cancelPayloadJSON: String?
    private var payloadJSON: String?
    private var error: Error?
    private var completed = false
    private var cancelHandler: (() -> Void)?
    let semaphore = DispatchSemaphore(value: 0)

    init(cancelPayloadJSON: String?) {
        self.cancelPayloadJSON = cancelPayloadJSON
        super.init()
    }

    func setCancelHandler(_ handler: @escaping () -> Void) {
        lock.lock()
        cancelHandler = handler
        lock.unlock()
    }

    func finish<T: Encodable>(_ payload: T) {
        do {
            complete(payloadJSON: try pkrEncodeJSON(payload), error: nil)
        } catch {
            complete(payloadJSON: nil, error: error)
        }
    }

    func cancelRequest() {
        lock.lock()
        let handler = cancelHandler
        lock.unlock()
        handler?()
    }

    func cancel() {
        cancelRequest()
        complete(payloadJSON: cancelPayloadJSON, error: nil)
    }

    func outcome() -> (payloadJSON: String?, error: Error?)? {
        lock.lock()
        defer { lock.unlock() }
        return completed ? (payloadJSON, error) : nil
    }

    private func complete(payloadJSON: String?, error: Error?) {
        lock.lock()
        guard !completed else {
            lock.unlock()
            return
        }
        completed = true
        self.payloadJSON = payloadJSON
        self.error = error
        lock.unlock()
        semaphore.signal()
    }
}

final class PKRResultSlot<Value> {
    private let lock = NSLock()
    private var result: Result<Value, Error>?
    let semaphore = DispatchSemaphore(value: 0)

    func fill(_ value: Result<Value, Error>) {
        lock.lock()
        guard result == nil else {
            lock.unlock()
            return
        }
        result = value
        lock.unlock()
        semaphore.signal()
    }

    func wait(timeoutMs: UInt64) -> Result<Value, Error>? {
        guard pkrWait(semaphore, timeoutMs: timeoutMs) else {
            return nil
        }
        lock.lock()
        defer { lock.unlock() }
        return result
    }
}

final class PKROnce {
    private let lock = NSLock()
    private var claimed = false

    func claim() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard !claimed else { return false }
        claimed = true
        return true
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

func pkrInfoFlag(_ info: [AnyHashable: Any], _ keys: String...) -> Bool {
    keys.contains { (info[$0] as? NSNumber)?.boolValue == true }
}

func pkrDeliversSingleResult(deliveryMode: String?, synchronous: Bool) -> Bool {
    synchronous || deliveryMode == "highQualityFormat" || deliveryMode == "fastFormat"
}

func pkrIsFinalImageResult(_ info: [AnyHashable: Any], hasResult: Bool, singleResult: Bool) -> Bool {
    if singleResult || pkrInfoFlag(info, PHImageCancelledKey, PHLivePhotoInfoCancelledKey) {
        return true
    }
    if info[PHImageErrorKey] != nil || info[PHLivePhotoInfoErrorKey] != nil {
        return true
    }
    if !hasResult && pkrInfoFlag(info, PHImageResultIsInCloudKey) {
        return true
    }
    return !pkrInfoFlag(info, PHImageResultIsDegradedKey, PHLivePhotoInfoIsDegradedKey)
}

func pkrImageResultPayload(_ image: NSImage?, info: [AnyHashable: Any]) -> PKRImageResultPayload {
    let tiffData = image?.tiffRepresentation
    return PKRImageResultPayload(
        tiffDataBase64: tiffData?.base64EncodedString() ?? "",
        width: tiffData == nil ? 0 : Double(image?.size.width ?? 0),
        height: tiffData == nil ? 0 : Double(image?.size.height ?? 0),
        cancelled: pkrInfoFlag(info, PHImageCancelledKey),
        degraded: pkrInfoFlag(info, PHImageResultIsDegradedKey),
        requestID: pkrRequestID(from: info),
        error: pkrResultErrorPayload(from: info, key: PHImageErrorKey)
    )
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
        cancelled: pkrInfoFlag(info, PHImageCancelledKey, PHLivePhotoInfoCancelledKey),
        degraded: pkrInfoFlag(info, PHImageResultIsDegradedKey, PHLivePhotoInfoIsDegradedKey),
        sizeWidth: livePhoto.map { Double($0.size.width) } ?? 0,
        sizeHeight: livePhoto.map { Double($0.size.height) } ?? 0,
        requestID: pkrRequestID(from: info),
        error: pkrResultErrorPayload(from: info, key: PHLivePhotoInfoErrorKey)
            ?? pkrResultErrorPayload(from: info, key: PHImageErrorKey)
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
        requestID: pkrRequestID(from: info),
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
    let slot = PKRResultSlot<T>()
    work({ slot.fill(.success($0)) }, { slot.fill(.failure($0)) })
    guard let result = slot.wait(timeoutMs: timeoutMs) else {
        cancel?()
        throw pkrError("request timed out")
    }
    return try result.get()
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
        let box = PKRRequestBox(
            cancelPayloadJSON: try? pkrEncodeJSON(
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
        )
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        let singleResult = pkrDeliversSingleResult(deliveryMode: request.deliveryMode, synchronous: request.synchronous)
        let requestID = imageManager.requestImage(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildImageRequestOptions(request)
        ) { image, info in
            let info = info ?? [:]
            guard pkrIsFinalImageResult(info, hasResult: image != nil, singleResult: singleResult) else {
                return
            }
            box.finish(pkrImageResultPayload(image, info: info))
        }
        box.setCancelHandler { imageManager.cancelImageRequest(requestID) }
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
        let box = PKRRequestBox(
            cancelPayloadJSON: try? pkrEncodeJSON(
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
        )
        let requestID = imageManager.requestImageDataAndOrientation(
            for: asset,
            options: pkrBuildImageRequestOptions(request)
        ) { imageData, dataUTI, orientation, info in
            let info = info ?? [:]
            box.finish(
                PKRImageDataResultPayload(
                    dataBase64: imageData?.base64EncodedString() ?? "",
                    uniformTypeIdentifier: dataUTI,
                    contentTypeIdentifier: nil,
                    orientation: Int32(clamping: orientation.rawValue),
                    cancelled: pkrInfoFlag(info, PHImageCancelledKey),
                    degraded: pkrInfoFlag(info, PHImageResultIsDegradedKey),
                    isInCloud: pkrInfoFlag(info, PHImageResultIsInCloudKey),
                    requestID: pkrRequestID(from: info),
                    error: pkrResultErrorPayload(from: info, key: PHImageErrorKey)
                )
            )
        }
        box.setCancelHandler { imageManager.cancelImageRequest(requestID) }
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
        let box = PKRRequestBox(
            cancelPayloadJSON: try? pkrEncodeJSON(
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
        )
        let targetSize = CGSize(width: request.targetWidth, height: request.targetHeight)
        let singleResult = pkrDeliversSingleResult(deliveryMode: request.deliveryMode, synchronous: false)
        let requestID = imageManager.requestLivePhoto(
            for: asset,
            targetSize: targetSize,
            contentMode: pkrContentMode(from: request.contentMode),
            options: pkrBuildLivePhotoRequestOptions(request)
        ) { livePhoto, info in
            let info = info ?? [:]
            guard pkrIsFinalImageResult(info, hasResult: livePhoto != nil, singleResult: singleResult) else {
                return
            }
            box.finish(pkrLivePhotoResultPayload(livePhoto, info: info))
        }
        box.setCancelHandler { imageManager.cancelImageRequest(requestID) }
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
    if box.outcome() == nil && !pkrWait(box.semaphore, timeoutMs: timeoutMs) {
        box.cancelRequest()
        pkrSetMessageError(outError, message: "request timed out")
        return nil
    }
    guard let outcome = box.outcome() else {
        pkrSetMessageError(outError, message: "request finished without a result")
        return nil
    }
    if let error = outcome.error {
        pkrSetError(outError, error)
        return nil
    }
    guard let payloadJSON = outcome.payloadJSON else {
        pkrSetMessageError(outError, message: "request finished without a result")
        return nil
    }
    return pkrCString(payloadJSON)
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
