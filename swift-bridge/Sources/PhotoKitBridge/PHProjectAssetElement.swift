import Foundation
import PhotosUI

struct PKRProjectAssetElementPayload: Codable {
    var weight: Double
    var placement: PKRRectPayload?
    var cloudAssetIdentifier: PKRCloudIdentifierPayload
    var annotation: String
    var cropRect: PKRRectPayload
    var regionsOfInterest: [PKRProjectRegionOfInterestPayload]
    var horizontallyFlipped: Bool
    var verticallyFlipped: Bool
}

func pkrEncodeProjectAssetElement(_ element: PHProjectAssetElement) -> PKRProjectAssetElementPayload {
    PKRProjectAssetElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement),
        cloudAssetIdentifier: pkrEncodeCloudIdentifier(element.cloudAssetIdentifier)!,
        annotation: element.annotation,
        cropRect: pkrRectPayload(element.cropRect),
        regionsOfInterest: element.regionsOfInterest.map(pkrEncodeProjectRegionOfInterest),
        horizontallyFlipped: element.horizontallyFlipped,
        verticallyFlipped: element.verticallyFlipped
    )
}

@_cdecl("ph_project_asset_element_is_available")
public func ph_project_asset_element_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
