import Foundation
import PhotosUI

struct PKRProjectElementPayload: Codable {
    var weight: Double
    var placement: PKRRectPayload?
}

func pkrEncodeProjectElement(_ element: PHProjectElement) -> PKRProjectElementPayload {
    PKRProjectElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement)
    )
}

@_cdecl("ph_project_element_is_available")
public func ph_project_element_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
