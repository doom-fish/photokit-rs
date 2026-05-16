import Foundation
import Photos
import UniformTypeIdentifiers

struct PKRContentEditingOutputPayload: Codable {
    var adjustmentData: PKRAdjustmentDataPayload?
    var renderedContentURL: String
    var defaultRenderedContentTypeIdentifier: String?
    var supportedRenderedContentTypeIdentifiers: [String]
}

final class PKRContentEditingOutputBox: NSObject {
    let output: PHContentEditingOutput

    init(output: PHContentEditingOutput) {
        self.output = output
        super.init()
    }
}

func pkrAdjustmentData(from payload: PKRAdjustmentDataPayload?) -> PHAdjustmentData? {
    guard let payload,
          let data = Data(base64Encoded: payload.dataBase64)
    else {
        return nil
    }
    return PHAdjustmentData(
        formatIdentifier: payload.formatIdentifier,
        formatVersion: payload.formatVersion,
        data: data
    )
}

func pkrEncodeContentEditingOutput(_ output: PHContentEditingOutput) -> PKRContentEditingOutputPayload {
    PKRContentEditingOutputPayload(
        adjustmentData: pkrEncodeAdjustmentData(output.adjustmentData),
        renderedContentURL: output.renderedContentURL.path,
        defaultRenderedContentTypeIdentifier: {
            if #available(macOS 14.0, *) {
                return output.defaultRenderedContentType?.identifier
            }
            return nil
        }(),
        supportedRenderedContentTypeIdentifiers: {
            if #available(macOS 14.0, *) {
                return output.supportedRenderedContentTypes.map(\.identifier)
            }
            return []
        }()
    )
}

@_cdecl("ph_content_editing_output_new_for_input")
public func ph_content_editing_output_new_for_input(
    _ input: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let input else {
        pkrSetMessageError(outError, message: "missing PHContentEditingInput")
        return nil
    }

    let contentEditingInput = pkrBorrow(input, as: PKRContentEditingInputBox.self).input
    return pkrRetain(PKRContentEditingOutputBox(output: PHContentEditingOutput(contentEditingInput: contentEditingInput)))
}

@_cdecl("ph_content_editing_output_json")
public func ph_content_editing_output_json(
    _ output: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let output else {
        pkrSetMessageError(outError, message: "missing PHContentEditingOutput")
        return nil
    }

    let contentEditingOutput = pkrBorrow(output, as: PKRContentEditingOutputBox.self).output
    return pkrCString(try! pkrEncodeJSON(pkrEncodeContentEditingOutput(contentEditingOutput)))
}

@_cdecl("ph_content_editing_output_release")
public func ph_content_editing_output_release(_ output: UnsafeMutableRawPointer?) {
    guard let output else { return }
    pkrRelease(output)
}

@_cdecl("ph_content_editing_output_set_adjustment_data_json")
public func ph_content_editing_output_set_adjustment_data_json(
    _ output: UnsafeMutableRawPointer?,
    _ adjustmentDataJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let output else {
        pkrSetMessageError(outError, message: "missing PHContentEditingOutput")
        return PKR_ERROR
    }

    do {
        let contentEditingOutput = pkrBorrow(output, as: PKRContentEditingOutputBox.self).output
        if let adjustmentDataJSON {
            let adjustmentDataPayload = try pkrDecodeJSON(adjustmentDataJSON, as: PKRAdjustmentDataPayload.self)
            contentEditingOutput.adjustmentData = pkrAdjustmentData(from: adjustmentDataPayload)
        } else {
            contentEditingOutput.adjustmentData = nil
        }
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_content_editing_output_rendered_content_url_for_type")
public func ph_content_editing_output_rendered_content_url_for_type(
    _ output: UnsafeMutableRawPointer?,
    _ typeIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let output else {
        pkrSetMessageError(outError, message: "missing PHContentEditingOutput")
        return nil
    }
    guard let typeIdentifier else {
        pkrSetMessageError(outError, message: "missing rendered content type identifier")
        return nil
    }

    guard #available(macOS 14.0, *) else {
        pkrSetMessageError(outError, message: "renderedContentURLForType requires macOS 14")
        return nil
    }

    do {
        guard let type = UTType(String(cString: typeIdentifier)) else {
            throw NSError(
                domain: "photokit-rs",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: "invalid rendered content type identifier"]
            )
        }
        let contentEditingOutput = pkrBorrow(output, as: PKRContentEditingOutputBox.self).output
        let url = try contentEditingOutput.renderedContentURL(for: type)
        return pkrCString(url.path)
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
