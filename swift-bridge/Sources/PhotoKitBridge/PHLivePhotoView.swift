import AppKit
import Foundation
import Photos
import PhotosUI

struct PKRLivePhotoViewPayload: Codable {
    var hasLivePhoto: Bool
    var livePhotoSizeWidth: Double?
    var livePhotoSizeHeight: Double?
    var contentMode: String
    var contentsRect: PKRRectPayload?
    var audioVolume: Float
    var muted: Bool
    var hasLivePhotoBadgeView: Bool
}

final class PKRLivePhotoViewBox: NSObject {
    let view: PHLivePhotoView

    init(view: PHLivePhotoView) {
        self.view = view
        super.init()
    }
}

func pkrLivePhotoViewContentMode(rawValue: Int32) throws -> PHLivePhotoViewContentMode {
    switch rawValue {
    case 0:
        return .aspectFit
    case 1:
        return .aspectFill
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported PHLivePhotoViewContentMode raw value: \(rawValue)"])
    }
}

func pkrLivePhotoViewPlaybackStyle(rawValue: Int32) throws -> PHLivePhotoViewPlaybackStyle {
    switch rawValue {
    case 0:
        return .undefined
    case 1:
        return .full
    case 2:
        return .hint
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported PHLivePhotoViewPlaybackStyle raw value: \(rawValue)"])
    }
}

func pkrEncodeLivePhotoViewPlaybackStyle(_ playbackStyle: PHLivePhotoViewPlaybackStyle) -> String {
    switch playbackStyle {
    case .undefined:
        return "undefined"
    case .full:
        return "full"
    case .hint:
        return "hint"
    @unknown default:
        return "undefined"
    }
}

func pkrEncodeLivePhotoView(_ view: PHLivePhotoView) -> PKRLivePhotoViewPayload {
    PKRLivePhotoViewPayload(
        hasLivePhoto: view.livePhoto != nil,
        livePhotoSizeWidth: view.livePhoto.map { Double($0.size.width) },
        livePhotoSizeHeight: view.livePhoto.map { Double($0.size.height) },
        contentMode: view.contentMode == .aspectFill ? "aspectFill" : "aspectFit",
        contentsRect: {
            if #available(macOS 14.0, *) {
                return pkrRectPayload(view.contentsRect)
            }
            return nil
        }(),
        audioVolume: view.audioVolume,
        muted: view.isMuted,
        hasLivePhotoBadgeView: view.livePhotoBadgeView != nil
    )
}

@_cdecl("ph_live_photo_view_is_available")
public func ph_live_photo_view_is_available() -> Int32 {
    if #available(macOS 10.12, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_live_photo_view_new")
public func ph_live_photo_view_new(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    if #available(macOS 10.12, *) {
        return pkrRetain(PKRLivePhotoViewBox(view: PHLivePhotoView(frame: .zero)))
    }
    pkrSetMessageError(outError, message: "PHLivePhotoView requires macOS 10.12")
    return nil
}

@_cdecl("ph_live_photo_view_release")
public func ph_live_photo_view_release(_ view: UnsafeMutableRawPointer?) {
    guard let view else { return }
    pkrRelease(view)
}

@_cdecl("ph_live_photo_view_json")
public func ph_live_photo_view_json(
    _ view: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return nil
    }

    do {
        return pkrCString(try pkrEncodeJSON(pkrEncodeLivePhotoView(pkrBorrow(view, as: PKRLivePhotoViewBox.self).view)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_view_set_content_mode")
public func ph_live_photo_view_set_content_mode(
    _ view: UnsafeMutableRawPointer?,
    _ contentMode: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    do {
        pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.contentMode = try pkrLivePhotoViewContentMode(rawValue: contentMode)
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_live_photo_view_set_contents_rect_json")
public func ph_live_photo_view_set_contents_rect_json(
    _ view: UnsafeMutableRawPointer?,
    _ rectJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }
    guard #available(macOS 14.0, *) else {
        pkrSetMessageError(outError, message: "PHLivePhotoView.contentsRect requires macOS 14.0")
        return PKR_ERROR
    }

    do {
        let rect = try pkrDecodeJSON(rectJSON, as: PKRRectPayload.self)
        pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.contentsRect = pkrRect(from: rect)
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_live_photo_view_set_audio_volume")
public func ph_live_photo_view_set_audio_volume(
    _ view: UnsafeMutableRawPointer?,
    _ audioVolume: Float,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.audioVolume = audioVolume
    return PKR_OK
}

@_cdecl("ph_live_photo_view_set_muted")
public func ph_live_photo_view_set_muted(
    _ view: UnsafeMutableRawPointer?,
    _ muted: Bool,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.isMuted = muted
    return PKR_OK
}

@_cdecl("ph_live_photo_view_clear_live_photo")
public func ph_live_photo_view_clear_live_photo(
    _ view: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.livePhoto = nil
    return PKR_OK
}

@_cdecl("ph_live_photo_view_request_with_resource_file_urls")
public func ph_live_photo_view_request_with_resource_file_urls(
    _ view: UnsafeMutableRawPointer?,
    _ fileURLsJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return nil
    }

    do {
        let fileURLs = try pkrDecodeJSON(fileURLsJSON, as: [String].self).map { value -> URL in
            if value.hasPrefix("file://") {
                return URL(string: value) ?? URL(fileURLWithPath: value)
            }
            return URL(fileURLWithPath: value)
        }
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let livePhotoView = pkrBorrow(view, as: PKRLivePhotoViewBox.self).view
        let semaphore = DispatchSemaphore(value: 0)
        var resultPayload = PKRLivePhotoResultPayload(
            hasLivePhoto: false,
            cancelled: false,
            degraded: false,
            sizeWidth: 0,
            sizeHeight: 0,
            requestID: nil,
            error: nil
        )
        var requestID: PHLivePhotoRequestID = 0
        requestID = PHLivePhoto.request(
            withResourceFileURLs: fileURLs,
            placeholderImage: nil,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode)
        ) { livePhoto, info in
            livePhotoView.livePhoto = livePhoto
            resultPayload = pkrLivePhotoResultPayload(livePhoto, info: info)
            semaphore.signal()
        }

        if semaphore.wait(timeout: .now() + .milliseconds(Int(timeoutMs))) == .timedOut {
            PHLivePhoto.cancelRequest(withRequestID: requestID)
            pkrSetMessageError(outError, message: "live photo view request timed out")
            return nil
        }

        return pkrCString(try pkrEncodeJSON(resultPayload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_view_start_playback")
public func ph_live_photo_view_start_playback(
    _ view: UnsafeMutableRawPointer?,
    _ playbackStyle: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    do {
        pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.startPlayback(with: try pkrLivePhotoViewPlaybackStyle(rawValue: playbackStyle))
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_live_photo_view_stop_playback")
public func ph_live_photo_view_stop_playback(
    _ view: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.stopPlayback()
    return PKR_OK
}

@_cdecl("ph_live_photo_view_stop_playback_animated")
public func ph_live_photo_view_stop_playback_animated(
    _ view: UnsafeMutableRawPointer?,
    _ animated: Bool,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return PKR_ERROR
    }

    pkrBorrow(view, as: PKRLivePhotoViewBox.self).view.stopPlayback(animated: animated)
    return PKR_OK
}
