import Foundation
import Photos
import PhotosUI

struct PKRPickerConfigurationPayload: Codable {
    var preferredAssetRepresentationMode: String
    var selection: String
    var selectionLimit: Int
    var filter: PKRPickerFilterPayload?
    var preselectedAssetIdentifiers: [String]
    var mode: String?
    var edgesWithoutContentMargins: UInt?
    var disabledCapabilities: UInt?
}

struct PKRPickerUpdateConfigurationPayload: Codable {
    var selectionLimit: Int?
    var edgesWithoutContentMargins: UInt?
}

func pkrPickerAssetRepresentationMode(from rawValue: String) throws -> PHPickerConfiguration.AssetRepresentationMode {
    switch rawValue {
    case "automatic":
        return .automatic
    case "current":
        return .current
    case "compatible":
        return .compatible
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported picker asset representation mode: \(rawValue)"])
    }
}

func pkrPickerSelection(from rawValue: String) throws -> PHPickerConfiguration.Selection {
    switch rawValue {
    case "default":
        return .default
    case "ordered":
        return .ordered
    case "continuous":
        if #available(macOS 14.0, *) {
            return .continuous
        }
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "continuous picker selection requires macOS 14.0"])
    case "continuousAndOrdered":
        if #available(macOS 14.0, *) {
            return .continuousAndOrdered
        }
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "continuous ordered picker selection requires macOS 14.0"])
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported picker selection: \(rawValue)"])
    }
}

@available(macOS 14.0, *)
func pkrPickerMode(from rawValue: String) throws -> PHPickerMode {
    switch rawValue {
    case "default":
        return .default
    case "compact":
        return .compact
    default:
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "unsupported picker mode: \(rawValue)"])
    }
}

func pkrBuildPickerConfiguration(
    from payload: PKRPickerConfigurationPayload,
    photoLibrary: PHPhotoLibrary?
) throws -> PHPickerConfiguration {
    var configuration = photoLibrary.map(PHPickerConfiguration.init(photoLibrary:)) ?? PHPickerConfiguration()
    configuration.preferredAssetRepresentationMode = try pkrPickerAssetRepresentationMode(from: payload.preferredAssetRepresentationMode)
    configuration.selection = try pkrPickerSelection(from: payload.selection)
    configuration.selectionLimit = payload.selectionLimit
    configuration.preselectedAssetIdentifiers = payload.preselectedAssetIdentifiers
    if let filter = payload.filter {
        configuration.filter = try pkrBuildPickerFilter(from: filter)
    }
    if #available(macOS 14.0, *) {
        if let mode = payload.mode {
            configuration.mode = try pkrPickerMode(from: mode)
        }
        if let edgesWithoutContentMargins = payload.edgesWithoutContentMargins {
            configuration.edgesWithoutContentMargins = NSDirectionalRectEdge(rawValue: edgesWithoutContentMargins)
        }
        if let disabledCapabilities = payload.disabledCapabilities {
            configuration.disabledCapabilities = PHPickerCapabilities(rawValue: disabledCapabilities)
        }
    } else if payload.mode != nil || payload.edgesWithoutContentMargins != nil || payload.disabledCapabilities != nil {
        throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "advanced picker configuration requires macOS 14.0"])
    }
    return configuration
}

@available(macOS 14.0, *)
func pkrBuildPickerUpdateConfiguration(
    from payload: PKRPickerUpdateConfigurationPayload
) throws -> PHPickerConfiguration.Update {
    var configuration = PHPickerConfiguration.Update()
    if let selectionLimit = payload.selectionLimit {
        configuration.selectionLimit = selectionLimit
    }
    if let edgesWithoutContentMargins = payload.edgesWithoutContentMargins {
        configuration.edgesWithoutContentMargins = NSDirectionalRectEdge(rawValue: edgesWithoutContentMargins)
    }
    return configuration
}

@_cdecl("ph_picker_configuration_is_available")
public func ph_picker_configuration_is_available() -> Int32 {
    if #available(macOS 13.0, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
