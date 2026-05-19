import Foundation
import Photos
import PhotosUI

final class PKRPickerViewControllerBox: NSObject {
    let controller: PHPickerViewController
    let configurationPayload: PKRPickerConfigurationPayload

    init(controller: PHPickerViewController, configurationPayload: PKRPickerConfigurationPayload) {
        self.controller = controller
        self.configurationPayload = configurationPayload
        super.init()
    }
}

@_cdecl("ph_picker_view_controller_is_available")
public func ph_picker_view_controller_is_available() -> Int32 {
    if #available(macOS 13.0, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_picker_view_controller_new")
public func ph_picker_view_controller_new(
    _ configurationJSON: UnsafePointer<CChar>?,
    _ photoLibrary: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let payload = try pkrDecodeJSON(configurationJSON, as: PKRPickerConfigurationPayload.self)
        let library = photoLibrary.map { pkrBorrow($0, as: PKRPhotoLibraryBox.self).library }
        let configuration = try pkrBuildPickerConfiguration(from: payload, photoLibrary: library)
        return pkrRetain(PKRPickerViewControllerBox(
            controller: PHPickerViewController(configuration: configuration),
            configurationPayload: payload
        ))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_picker_view_controller_release")
public func ph_picker_view_controller_release(_ controller: UnsafeMutableRawPointer?) {
    guard let controller else { return }
    pkrRelease(controller)
}

@_cdecl("ph_picker_view_controller_update_picker_json")
public func ph_picker_view_controller_update_picker_json(
    _ controller: UnsafeMutableRawPointer?,
    _ configurationJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return PKR_ERROR
    }

    do {
        guard #available(macOS 14.0, *) else {
            throw NSError(domain: "photokit-rs", code: -1, userInfo: [NSLocalizedDescriptionKey: "picker updates require macOS 14.0"])
        }
        let payload = try pkrDecodeJSON(configurationJSON, as: PKRPickerUpdateConfigurationPayload.self)
        let configuration = try pkrBuildPickerUpdateConfiguration(from: payload)
        pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.updatePicker(using: configuration)
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_picker_view_controller_deselect_assets_json")
public func ph_picker_view_controller_deselect_assets_json(
    _ controller: UnsafeMutableRawPointer?,
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return PKR_ERROR
    }

    do {
        let identifiers = try pkrDecodeJSON(identifiersJSON, as: [String].self)
        pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.deselectAssets(withIdentifiers: identifiers)
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_picker_view_controller_move_asset")
public func ph_picker_view_controller_move_asset(
    _ controller: UnsafeMutableRawPointer?,
    _ identifier: UnsafePointer<CChar>?,
    _ afterIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller, let identifier else {
        pkrSetMessageError(outError, message: "missing picker move identifiers")
        return PKR_ERROR
    }

    pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.moveAsset(
        withIdentifier: String(cString: identifier),
        afterAssetWithIdentifier: afterIdentifier.map { String(cString: $0) }
    )
    return PKR_OK
}

@_cdecl("ph_picker_view_controller_scroll_to_initial_position")
public func ph_picker_view_controller_scroll_to_initial_position(
    _ controller: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return PKR_ERROR
    }

    if #available(macOS 14.0, *) {
        pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.scrollToInitialPosition()
        return PKR_OK
    }
    pkrSetMessageError(outError, message: "scrollToInitialPosition requires macOS 14.0")
    return PKR_ERROR
}

@_cdecl("ph_picker_view_controller_zoom_in")
public func ph_picker_view_controller_zoom_in(
    _ controller: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return PKR_ERROR
    }

    if #available(macOS 14.0, *) {
        pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.zoomIn()
        return PKR_OK
    }
    pkrSetMessageError(outError, message: "zoomIn requires macOS 14.0")
    return PKR_ERROR
}

@_cdecl("ph_picker_view_controller_zoom_out")
public func ph_picker_view_controller_zoom_out(
    _ controller: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return PKR_ERROR
    }

    if #available(macOS 14.0, *) {
        pkrBorrow(controller, as: PKRPickerViewControllerBox.self).controller.zoomOut()
        return PKR_OK
    }
    pkrSetMessageError(outError, message: "zoomOut requires macOS 14.0")
    return PKR_ERROR
}
