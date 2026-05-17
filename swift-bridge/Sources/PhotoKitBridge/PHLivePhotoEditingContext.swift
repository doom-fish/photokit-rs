import CoreImage
import Foundation
import Photos

public typealias PKRLivePhotoFrameProcessorCallback = @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Int32

final class PKRLivePhotoEditingContextBox: NSObject {
    let context: PHLivePhotoEditingContext

    init(context: PHLivePhotoEditingContext) {
        self.context = context
        super.init()
    }
}

struct PKRLivePhotoEditingContextPayload: Codable {
    var fullSizeImageWidth: Double
    var fullSizeImageHeight: Double
    var durationSeconds: Double
    var photoTimeSeconds: Double
    var audioVolume: Float
    var orientation: Int32
}

struct PKRLivePhotoFramePayload: Codable {
    var frameType: Int
    var timeSeconds: Double
    var renderScale: Double
    var imageWidth: Double
    var imageHeight: Double
}

struct PKRLivePhotoEditingSaveResultPayload: Codable {
    var success: Bool
}

func pkrEncodeLivePhotoEditingContext(_ context: PHLivePhotoEditingContext) -> PKRLivePhotoEditingContextPayload {
    PKRLivePhotoEditingContextPayload(
        fullSizeImageWidth: context.fullSizeImage.extent.width,
        fullSizeImageHeight: context.fullSizeImage.extent.height,
        durationSeconds: CMTimeGetSeconds(context.duration),
        photoTimeSeconds: CMTimeGetSeconds(context.photoTime),
        audioVolume: context.audioVolume,
        orientation: Int32(context.orientation.rawValue)
    )
}

func pkrEncodeLivePhotoFrame(_ frame: PHLivePhotoFrame) -> PKRLivePhotoFramePayload {
    PKRLivePhotoFramePayload(
        frameType: frame.type.rawValue,
        timeSeconds: CMTimeGetSeconds(frame.time),
        renderScale: Double(frame.renderScale),
        imageWidth: frame.image.extent.width,
        imageHeight: frame.image.extent.height
    )
}

@_cdecl("ph_live_photo_editing_context_new")
public func ph_live_photo_editing_context_new(
    _ input: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let input else {
        pkrSetMessageError(outError, message: "missing PHContentEditingInput")
        return nil
    }
    do {
        let livePhotoInput = pkrBorrow(input, as: PHContentEditingInput.self)
        guard let context = PHLivePhotoEditingContext(livePhotoEditingInput: livePhotoInput) else {
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "content editing input is not for a live photo"])
        }
        return pkrRetain(PKRLivePhotoEditingContextBox(context: context))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_editing_context_release")
public func ph_live_photo_editing_context_release(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    pkrRelease(context)
}

@_cdecl("ph_live_photo_editing_context_json")
public func ph_live_photo_editing_context_json(
    _ context: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoEditingContext")
        return nil
    }
    do {
        let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
        return pkrCString(try pkrEncodeJSON(pkrEncodeLivePhotoEditingContext(editingContext)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_editing_context_set_audio_volume")
public func ph_live_photo_editing_context_set_audio_volume(
    _ context: UnsafeMutableRawPointer?,
    _ audioVolume: Float,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoEditingContext")
        return PKR_ERROR
    }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    editingContext.audioVolume = audioVolume
    return PKR_OK
}

@_cdecl("ph_live_photo_editing_context_set_frame_processor")
public func ph_live_photo_editing_context_set_frame_processor(
    _ context: UnsafeMutableRawPointer?,
    _ callback: @escaping PKRLivePhotoFrameProcessorCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoEditingContext")
        return PKR_ERROR
    }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    editingContext.frameProcessor = { frame, _ in
        guard let json = try? pkrEncodeJSON(pkrEncodeLivePhotoFrame(frame)) else {
            return frame.image
        }
        let decision = json.withCString { callback($0, userInfo) }
        switch decision {
        case 1:
            return nil
        default:
            return frame.image
        }
    }
    return PKR_OK
}

@_cdecl("ph_live_photo_editing_context_clear_frame_processor")
public func ph_live_photo_editing_context_clear_frame_processor(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    editingContext.frameProcessor = nil
}

@_cdecl("ph_live_photo_editing_context_prepare_live_photo_json")
public func ph_live_photo_editing_context_prepare_live_photo_json(
    _ context: UnsafeMutableRawPointer?,
    _ targetWidth: Double,
    _ targetHeight: Double,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoEditingContext")
        return nil
    }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    do {
        let semaphore = DispatchSemaphore(value: 0)
        var payload: PKRLivePhotoResultPayload?
        var requestError: Error?
        editingContext.prepareLivePhotoForPlayback(withTargetSize: CGSize(width: targetWidth, height: targetHeight), options: nil) { livePhoto, error in
            if let error {
                requestError = error
            }
            payload = PKRLivePhotoResultPayload(
                hasLivePhoto: livePhoto != nil,
                cancelled: false,
                degraded: false,
                sizeWidth: Double(livePhoto?.size.width ?? 0),
                sizeHeight: Double(livePhoto?.size.height ?? 0),
                requestID: nil,
                error: error.map(pkrErrorPayload)
            )
            semaphore.signal()
        }
        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        guard semaphore.wait(timeout: timeout) == .success else {
            editingContext.cancel()
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "live photo playback preparation timed out"])
        }
        if let requestError, payload == nil {
            throw requestError
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_editing_context_save_json")
public func ph_live_photo_editing_context_save_json(
    _ context: UnsafeMutableRawPointer?,
    _ output: UnsafeMutableRawPointer?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoEditingContext")
        return nil
    }
    guard let output else {
        pkrSetMessageError(outError, message: "missing PHContentEditingOutput")
        return nil
    }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    let editingOutput = pkrBorrow(output, as: PHContentEditingOutput.self)
    do {
        let semaphore = DispatchSemaphore(value: 0)
        var payload = PKRLivePhotoEditingSaveResultPayload(success: false)
        editingContext.saveLivePhoto(to: editingOutput, options: nil) { success, error in
            payload = PKRLivePhotoEditingSaveResultPayload(success: success)
            if let error {
                pkrSetError(outError, error)
            }
            semaphore.signal()
        }
        let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMs))
        guard semaphore.wait(timeout: timeout) == .success else {
            editingContext.cancel()
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "live photo save timed out"])
        }
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_live_photo_editing_context_cancel")
public func ph_live_photo_editing_context_cancel(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    let editingContext = pkrBorrow(context, as: PKRLivePhotoEditingContextBox.self).context
    editingContext.cancel()
}
