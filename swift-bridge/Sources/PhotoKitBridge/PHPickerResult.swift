import Foundation
import PhotosUI

struct PKRPickerItemProviderPayload: Codable {
    var registeredTypeIdentifiers: [String]
    var suggestedName: String?
}

struct PKRPickerResultPayload: Codable {
    var assetIdentifier: String?
    var itemProvider: PKRPickerItemProviderPayload
}

func pkrEncodePickerResult(_ result: PHPickerResult) -> PKRPickerResultPayload {
    PKRPickerResultPayload(
        assetIdentifier: result.assetIdentifier,
        itemProvider: PKRPickerItemProviderPayload(
            registeredTypeIdentifiers: result.itemProvider.registeredTypeIdentifiers,
            suggestedName: result.itemProvider.suggestedName
        )
    )
}

@_cdecl("ph_picker_result_is_available")
public func ph_picker_result_is_available() -> Int32 {
    if #available(macOS 13.0, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
