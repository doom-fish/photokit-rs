import Foundation
import PhotosUI

@_cdecl("ph_project_extension_controller_is_available")
public func ph_project_extension_controller_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
