import AppKit
import Foundation
import Photos

@_cdecl("ph_live_photo_request_with_resource_file_urls")
public func ph_live_photo_request_with_resource_file_urls(
    _ fileURLsJSON: UnsafePointer<CChar>?,
    _ requestJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let fileURLs = try pkrDecodeJSON(fileURLsJSON, as: [String].self).map { value -> URL in
            if value.hasPrefix("file://") {
                return URL(string: value) ?? URL(fileURLWithPath: value)
            }
            return URL(fileURLWithPath: value)
        }
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let box = PKRRequestBox()
        box.cancelPayloadJSON = try? pkrEncodeJSON(
            PKRLivePhotoResultPayload(
                hasLivePhoto: false,
                cancelled: true,
                degraded: false,
                sizeWidth: 0,
                sizeHeight: 0
            )
        )
        box.requestID = PHLivePhoto.request(
            withResourceFileURLs: fileURLs,
            placeholderImage: nil,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode)
        ) { livePhoto, info in
            box.finish(pkrLivePhotoResultPayload(livePhoto, info: info))
        }
        box.cancelHandler = { PHLivePhoto.cancelRequest(withRequestID: PHLivePhotoRequestID(box.requestID)) }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
