import Foundation
import Photos
import PhotosUI

struct PKRPickerFilterPayload: Codable {
    var kind: String
    var playbackStyle: PKRAssetPlaybackStyle?
    var subfilters: [PKRPickerFilterPayload]?
}

func pkrPlaybackStyle(from payload: PKRAssetPlaybackStyle) -> PHAsset.PlaybackStyle {
    switch payload {
    case .unsupported:
        return .unsupported
    case .image:
        return .image
    case .imageAnimated:
        return .imageAnimated
    case .livePhoto:
        return .livePhoto
    case .video:
        return .video
    case .videoLooping:
        return .videoLooping
    }
}

func pkrBuildPickerFilter(from payload: PKRPickerFilterPayload) throws -> PHPickerFilter {
    switch payload.kind {
    case "images":
        return .images
    case "videos":
        return .videos
    case "livePhotos":
        return .livePhotos
    case "depthEffectPhotos":
        return .depthEffectPhotos
    case "bursts":
        return .bursts
    case "panoramas":
        return .panoramas
    case "screenshots":
        return .screenshots
    case "screenRecordings":
        return .screenRecordings
    case "cinematicVideos":
        return .cinematicVideos
    case "slomoVideos":
        return .slomoVideos
    case "timelapseVideos":
        return .timelapseVideos
    case "spatialMedia":
        if #available(macOS 15.0, *) {
            return .spatialMedia
        }
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "spatial media filter requires macOS 15.0"])
    case "playbackStyle":
        guard let playbackStyle = payload.playbackStyle else {
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "missing picker playback style"])
        }
        return .playbackStyle(pkrPlaybackStyle(from: playbackStyle))
    case "any":
        return .any(of: try payload.subfilters?.map(pkrBuildPickerFilter) ?? [])
    case "all":
        return .all(of: try payload.subfilters?.map(pkrBuildPickerFilter) ?? [])
    case "not":
        guard let subfilter = payload.subfilters?.first else {
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "missing picker subfilter"])
        }
        return .not(try pkrBuildPickerFilter(from: subfilter))
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported PHPickerFilter payload kind: \(payload.kind)"])
    }
}

@_cdecl("ph_picker_filter_is_available")
public func ph_picker_filter_is_available() -> Int32 {
    if #available(macOS 13.0, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_picker_filter_description_json")
public func ph_picker_filter_description_json(
    _ filterJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    do {
        let payload = try pkrDecodeJSON(filterJSON, as: PKRPickerFilterPayload.self)
        let filter = try pkrBuildPickerFilter(from: payload)
        return pkrCString(String(describing: filter))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
