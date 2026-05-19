import Foundation
import PhotosUI

public typealias PKRPickerViewControllerDelegateCallback = @convention(c) (UnsafeMutablePointer<CChar>?, UnsafeMutableRawPointer?) -> Void

final class PKRPickerViewControllerDelegateBox: NSObject, PHPickerViewControllerDelegate {
    weak var controller: PHPickerViewController?
    let callback: PKRPickerViewControllerDelegateCallback
    let userInfo: UnsafeMutableRawPointer?

    init(
        controller: PHPickerViewController,
        callback: @escaping PKRPickerViewControllerDelegateCallback,
        userInfo: UnsafeMutableRawPointer?
    ) {
        self.controller = controller
        self.callback = callback
        self.userInfo = userInfo
        super.init()
    }

    func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
        let json = try? pkrEncodeJSON(results.map(pkrEncodePickerResult))
        callback(json.flatMap(pkrCString), userInfo)
    }
}

@_cdecl("ph_picker_view_controller_delegate_is_available")
public func ph_picker_view_controller_delegate_is_available() -> Int32 {
    if #available(macOS 13.0, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_picker_view_controller_register_delegate")
public func ph_picker_view_controller_register_delegate(
    _ controller: UnsafeMutableRawPointer?,
    _ callback: @escaping PKRPickerViewControllerDelegateCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let controller else {
        pkrSetMessageError(outError, message: "missing PHPickerViewController")
        return nil
    }

    let box = pkrBorrow(controller, as: PKRPickerViewControllerBox.self)
    let delegate = PKRPickerViewControllerDelegateBox(
        controller: box.controller,
        callback: callback,
        userInfo: userInfo
    )
    box.controller.delegate = delegate
    return pkrRetain(delegate)
}

@_cdecl("ph_picker_view_controller_unregister_delegate")
public func ph_picker_view_controller_unregister_delegate(_ delegate: UnsafeMutableRawPointer?) {
    guard let delegate else { return }
    let box = pkrBorrow(delegate, as: PKRPickerViewControllerDelegateBox.self)
    box.controller?.delegate = nil
    pkrRelease(delegate)
}
