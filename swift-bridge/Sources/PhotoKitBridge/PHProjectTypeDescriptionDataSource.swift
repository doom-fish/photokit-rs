import Foundation
import PhotosUI

@_cdecl("ph_project_type_description_data_source_is_available")
public func ph_project_type_description_data_source_is_available() -> Int32 {
    if #available(macOS 10.14, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
