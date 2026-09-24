import Foundation
import PhotosUI

struct PKRLivePhotoViewDelegateEventPayload: Codable {
    var kind: String
    var playbackStyle: String
}

public typealias PKRLivePhotoViewDelegateCallback = @convention(c) (UnsafeMutablePointer<CChar>?, UnsafeMutableRawPointer?) -> Int32

final class PKRLivePhotoViewDelegateBox: NSObject, PHLivePhotoViewDelegate {
    weak var view: PHLivePhotoView?
    let callback: PKRLivePhotoViewDelegateCallback
    let userInfo: UnsafeMutableRawPointer?
    private let contextRelease: PKRObserverContextCallback?

    init(
        view: PHLivePhotoView,
        callback: @escaping PKRLivePhotoViewDelegateCallback,
        userInfo: UnsafeMutableRawPointer?,
        contextRetain: PKRObserverContextCallback?,
        contextRelease: PKRObserverContextCallback?
    ) {
        self.view = view
        self.callback = callback
        self.userInfo = userInfo
        self.contextRelease = contextRelease
        super.init()
        if let userInfo {
            contextRetain?(userInfo)
        }
    }

    deinit {
        if let userInfo {
            contextRelease?(userInfo)
        }
    }

    private func sendEvent(kind: String, playbackStyle: PHLivePhotoViewPlaybackStyle) -> Int32 {
        let payload = PKRLivePhotoViewDelegateEventPayload(
            kind: kind,
            playbackStyle: pkrEncodeLivePhotoViewPlaybackStyle(playbackStyle)
        )
        let json = try? pkrEncodeJSON(payload)
        return callback(json.flatMap(pkrCString), userInfo)
    }

    func livePhotoView(_ livePhotoView: PHLivePhotoView, canBeginPlaybackWith playbackStyle: PHLivePhotoViewPlaybackStyle) -> Bool {
        sendEvent(kind: "canBegin", playbackStyle: playbackStyle) != 0
    }

    func livePhotoView(_ livePhotoView: PHLivePhotoView, willBeginPlaybackWith playbackStyle: PHLivePhotoViewPlaybackStyle) {
        _ = sendEvent(kind: "willBegin", playbackStyle: playbackStyle)
    }

    func livePhotoView(_ livePhotoView: PHLivePhotoView, didEndPlaybackWith playbackStyle: PHLivePhotoViewPlaybackStyle) {
        _ = sendEvent(kind: "didEnd", playbackStyle: playbackStyle)
    }
}

@_cdecl("ph_live_photo_view_delegate_is_available")
public func ph_live_photo_view_delegate_is_available() -> Int32 {
    if #available(macOS 10.12, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_live_photo_view_register_delegate")
public func ph_live_photo_view_register_delegate(
    _ view: UnsafeMutableRawPointer?,
    _ callback: @escaping PKRLivePhotoViewDelegateCallback,
    _ userInfo: UnsafeMutableRawPointer?,
    _ contextRetain: @escaping PKRObserverContextCallback,
    _ contextRelease: @escaping PKRObserverContextCallback,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let view else {
        pkrSetMessageError(outError, message: "missing PHLivePhotoView")
        return nil
    }

    let livePhotoView = pkrBorrow(view, as: PKRLivePhotoViewBox.self).view
    let delegate = PKRLivePhotoViewDelegateBox(
        view: livePhotoView,
        callback: callback,
        userInfo: userInfo,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )
    livePhotoView.delegate = delegate
    return pkrRetain(delegate)
}

@_cdecl("ph_live_photo_view_unregister_delegate")
public func ph_live_photo_view_unregister_delegate(_ delegate: UnsafeMutableRawPointer?) {
    guard let delegate else { return }
    let box = pkrBorrow(delegate, as: PKRLivePhotoViewDelegateBox.self)
    box.view?.delegate = nil
    pkrRelease(delegate)
}
