import Foundation
import Photos
import PhotosUI

@_cdecl("ph_project_extension_context_is_available")
public func ph_project_extension_context_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}

@_cdecl("ph_project_extension_context_release")
public func ph_project_extension_context_release(_ context: UnsafeMutableRawPointer?) {
    guard let context else { return }
    pkrRelease(context)
}

@_cdecl("ph_project_extension_context_project_json")
public func ph_project_extension_context_project_json(
    _ context: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHProjectExtensionContext")
        return nil
    }

    do {
        let extensionContext = pkrBorrow(context, as: PHProjectExtensionContext.self)
        return pkrCString(try pkrEncodeJSON(pkrEncodeProject(extensionContext.project)))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_project_extension_context_photo_library")
public func ph_project_extension_context_photo_library(
    _ context: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHProjectExtensionContext")
        return nil
    }

    let extensionContext = pkrBorrow(context, as: PHProjectExtensionContext.self)
    return pkrRetain(PKRPhotoLibraryBox(library: extensionContext.photoLibrary))
}

@_cdecl("ph_project_extension_context_show_editor_for_asset")
public func ph_project_extension_context_show_editor_for_asset(
    _ context: UnsafeMutableRawPointer?,
    _ assetIdentifier: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let context, let assetIdentifier else {
        pkrSetMessageError(outError, message: "missing project extension editor inputs")
        return PKR_ERROR
    }

    do {
        let extensionContext = pkrBorrow(context, as: PHProjectExtensionContext.self)
        let asset = try pkrRequestAsset(localIdentifier: String(cString: assetIdentifier))
        extensionContext.showEditor(for: asset)
        return PKR_OK
    } catch {
        pkrSetError(outError, error)
        return PKR_ERROR
    }
}

@_cdecl("ph_project_extension_context_updated_project_info_json")
public func ph_project_extension_context_updated_project_info_json(
    _ context: UnsafeMutableRawPointer?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let context else {
        pkrSetMessageError(outError, message: "missing PHProjectExtensionContext")
        return nil
    }

    let extensionContext = pkrBorrow(context, as: PHProjectExtensionContext.self)
    let semaphore = DispatchSemaphore(value: 0)
    var updatedProjectInfo: PHProjectInfo?
    let progress = extensionContext.updatedProjectInfo(from: nil) { projectInfo in
        updatedProjectInfo = projectInfo
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + .milliseconds(Int(timeoutMs))) == .timedOut {
        progress.cancel()
        pkrSetMessageError(outError, message: "updated project info request timed out")
        return nil
    }

    do {
        let payload = updatedProjectInfo.map(pkrEncodeProjectInfo)
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
