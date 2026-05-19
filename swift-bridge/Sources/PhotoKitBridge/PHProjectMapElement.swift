import Foundation
import MapKit
import PhotosUI

struct PKRProjectMapAnnotationPayload: Codable {
    var coordinate: PKRCoordinatePayload
    var title: String?
    var subtitle: String?
}

struct PKRProjectMapElementPayload: Codable {
    var weight: Double
    var placement: PKRRectPayload?
    var mapType: UInt64
    var centerCoordinate: PKRCoordinatePayload
    var heading: Double
    var pitch: Double
    var altitude: Double
    var annotations: [PKRProjectMapAnnotationPayload]
}

func pkrEncodeProjectMapAnnotation(_ annotation: MKAnnotation) -> PKRProjectMapAnnotationPayload {
    PKRProjectMapAnnotationPayload(
        coordinate: PKRCoordinatePayload(
            latitude: annotation.coordinate.latitude,
            longitude: annotation.coordinate.longitude
        ),
        title: annotation.title ?? nil,
        subtitle: annotation.subtitle ?? nil
    )
}

func pkrEncodeProjectMapElement(_ element: PHProjectMapElement) -> PKRProjectMapElementPayload {
    PKRProjectMapElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement),
        mapType: UInt64(element.mapType.rawValue),
        centerCoordinate: PKRCoordinatePayload(
            latitude: element.centerCoordinate.latitude,
            longitude: element.centerCoordinate.longitude
        ),
        heading: element.heading,
        pitch: Double(element.pitch),
        altitude: element.altitude,
        annotations: element.annotations.map(pkrEncodeProjectMapAnnotation)
    )
}

@_cdecl("ph_project_map_element_is_available")
public func ph_project_map_element_is_available() -> Int32 {
    if #available(macOS 10.14, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
