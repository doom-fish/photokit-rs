import Foundation
import PhotosUI

@_cdecl("ph_content_editing_controller_is_available")
public func ph_content_editing_controller_is_available() -> Int32 {
    if #available(macOS 10.11, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
