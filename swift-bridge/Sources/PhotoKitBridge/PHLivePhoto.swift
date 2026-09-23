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
        let fileURLs = try pkrDecodeJSON(fileURLsJSON, as: [String].self).map(pkrFileURL)
        let request = try pkrDecodeJSON(requestJSON, as: PKRImageRequestPayload.self)
        let box = PKRRequestBox(
            cancelPayloadJSON: try? pkrEncodeJSON(
                PKRLivePhotoResultPayload(
                    hasLivePhoto: false,
                    cancelled: true,
                    degraded: false,
                    sizeWidth: 0,
                    sizeHeight: 0
                )
            )
        )
        let requestID = PHLivePhoto.request(
            withResourceFileURLs: fileURLs,
            placeholderImage: nil,
            targetSize: CGSize(width: request.targetWidth, height: request.targetHeight),
            contentMode: pkrContentMode(from: request.contentMode)
        ) { livePhoto, info in
            guard pkrIsFinalImageResult(info, hasResult: livePhoto != nil, singleResult: false) else {
                return
            }
            box.finish(pkrLivePhotoResultPayload(livePhoto, info: info))
        }
        box.setCancelHandler { PHLivePhoto.cancelRequest(withRequestID: requestID) }
        return pkrRetain(box)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
