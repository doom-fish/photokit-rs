import Foundation
import PhotosUI

struct PKRProjectRegionOfInterestPayload: Codable {
    var rect: PKRRectPayload
    var weight: Double
    var quality: Double
    var identifier: String
}

func pkrEncodeProjectRegionOfInterest(_ regionOfInterest: PHProjectRegionOfInterest) -> PKRProjectRegionOfInterestPayload {
    PKRProjectRegionOfInterestPayload(
        rect: pkrRectPayload(regionOfInterest.rect),
        weight: regionOfInterest.weight,
        quality: regionOfInterest.quality,
        identifier: regionOfInterest.identifier.rawValue
    )
}

@_cdecl("ph_project_region_of_interest_is_available")
public func ph_project_region_of_interest_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
