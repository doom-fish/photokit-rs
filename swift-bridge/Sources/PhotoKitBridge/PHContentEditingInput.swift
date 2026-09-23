import AVFoundation
import AppKit
import Foundation
import Photos

struct PKRAdjustmentDataPayload: Codable {
    var formatIdentifier: String
    var formatVersion: String
    var dataBase64: String
}

struct PKRContentEditingInputRequestOptionsPayload: Codable {
    var networkAccessAllowed: Bool
    var acceptsAnyAdjustmentData: Bool
}

struct PKRContentEditingInputPayload: Codable {
    var mediaType: PKRMediaType
    var mediaSubtypes: UInt64
    var creationDate: String?
    var location: PKRCoordinatePayload?
    var contentTypeIdentifier: String?
    var uniformTypeIdentifier: String?
    var playbackStyle: PKRAssetPlaybackStyle?
    var adjustmentData: PKRAdjustmentDataPayload?
    var hasDisplaySizeImage: Bool
    var displaySizeImageWidth: Double?
    var displaySizeImageHeight: Double?
    var fullSizeImageURL: String?
    var fullSizeImageOrientation: Int32
    var audiovisualAssetClass: String?
    var hasLivePhoto: Bool
    var livePhotoSizeWidth: Double?
    var livePhotoSizeHeight: Double?
}

final class PKRContentEditingInputBox: NSObject {
    let input: PHContentEditingInput

    init(input: PHContentEditingInput) {
        self.input = input
        super.init()
    }
}

func pkrEncodeAdjustmentData(_ adjustmentData: PHAdjustmentData?) -> PKRAdjustmentDataPayload? {
    guard let adjustmentData else { return nil }
    return PKRAdjustmentDataPayload(
        formatIdentifier: adjustmentData.formatIdentifier,
        formatVersion: adjustmentData.formatVersion,
        dataBase64: adjustmentData.data.base64EncodedString()
    )
}

func pkrEncodeContentEditingInput(_ input: PHContentEditingInput) -> PKRContentEditingInputPayload {
    let displaySizeImageSize = input.displaySizeImage?.size
    return PKRContentEditingInputPayload(
        mediaType: pkrMediaType(from: input.mediaType),
        mediaSubtypes: UInt64(input.mediaSubtypes.rawValue),
        creationDate: pkrDateString(input.creationDate),
        location: pkrCoordinatePayload(input.location),
        contentTypeIdentifier: {
            if #available(macOS 26.0, *) {
                return input.contentType?.identifier
            }
            return nil
        }(),
        uniformTypeIdentifier: input.uniformTypeIdentifier,
        playbackStyle: {
            if #available(macOS 10.13, *) {
                return pkrAssetPlaybackStyle(from: input.playbackStyle)
            }
            return nil
        }(),
        adjustmentData: pkrEncodeAdjustmentData(input.adjustmentData),
        hasDisplaySizeImage: input.displaySizeImage != nil,
        displaySizeImageWidth: displaySizeImageSize.map { Double($0.width) },
        displaySizeImageHeight: displaySizeImageSize.map { Double($0.height) },
        fullSizeImageURL: input.fullSizeImageURL?.path,
        fullSizeImageOrientation: Int32(input.fullSizeImageOrientation),
        audiovisualAssetClass: input.audiovisualAsset.map { String(describing: type(of: $0)) },
        hasLivePhoto: input.livePhoto != nil,
        livePhotoSizeWidth: input.livePhoto.map { Double($0.size.width) },
        livePhotoSizeHeight: input.livePhoto.map { Double($0.size.height) }
    )
}

@_cdecl("ph_asset_request_content_editing_input")
public func ph_asset_request_content_editing_input(
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing asset identifier")
        return nil
    }

    do {
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        let payload = try pkrDecodeJSON(optionsJSON, as: PKRContentEditingInputRequestOptionsPayload.self)
        let options = PHContentEditingInputRequestOptions()
        options.isNetworkAccessAllowed = payload.networkAccessAllowed
        options.canHandleAdjustmentData = { _ in payload.acceptsAnyAdjustmentData }

        let slot = PKRResultSlot<PHContentEditingInput?>()
        let requestID = asset.requestContentEditingInput(with: options) { input, info in
            if let error = info[PHContentEditingInputErrorKey] as? Error {
                slot.fill(.failure(error))
            } else if pkrInfoFlag(info, PHContentEditingInputCancelledKey) {
                slot.fill(.failure(pkrError("content editing input request cancelled")))
            } else if pkrInfoFlag(info, PHContentEditingInputResultIsInCloudKey) {
                slot.fill(.failure(pkrError("content editing input is in iCloud")))
            } else {
                slot.fill(.success(input))
            }
        }

        guard let outcome = slot.wait(timeoutMs: timeoutMs) else {
            asset.cancelContentEditingInputRequest(requestID)
            pkrSetMessageError(outError, message: "content editing input request timed out")
            return nil
        }
        guard let result = try outcome.get() else {
            pkrSetMessageError(outError, message: "missing PHContentEditingInput result")
            return nil
        }
        return pkrRetain(PKRContentEditingInputBox(input: result))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_content_editing_input_json")
public func ph_content_editing_input_json(
    _ input: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let input else {
        pkrSetMessageError(outError, message: "missing PHContentEditingInput")
        return nil
    }

    let contentEditingInput = pkrBorrow(input, as: PKRContentEditingInputBox.self).input
    do {
        return pkrCString(try pkrEncodeJSON(pkrEncodeContentEditingInput(contentEditingInput)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_content_editing_input_release")
public func ph_content_editing_input_release(_ input: UnsafeMutableRawPointer?) {
    guard let input else { return }
    pkrRelease(input)
}
