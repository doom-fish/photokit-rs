import Foundation
import Photos

struct PKRAssetResourceRequestOptionsPayload: Codable {
    var networkAccessAllowed: Bool
}

public typealias PKRDataChunkCallback = @convention(c) (UnsafeRawPointer?, Int, UnsafeMutableRawPointer?) -> Void

struct PKRAssetResourceDataRequestPayload: Codable {
    var requestID: Int32
    var error: PKRErrorPayload?
}

final class PKRDataSink {
    private let callback: PKRDataChunkCallback
    private let context: UnsafeMutableRawPointer
    private let contextRelease: PKRObserverContextCallback

    init(
        callback: @escaping PKRDataChunkCallback,
        context: UnsafeMutableRawPointer,
        contextRetain: PKRObserverContextCallback,
        contextRelease: @escaping PKRObserverContextCallback
    ) {
        self.callback = callback
        self.context = context
        self.contextRelease = contextRelease
        contextRetain(context)
    }

    deinit {
        contextRelease(context)
    }

    func append(_ data: Data) {
        data.withUnsafeBytes { buffer in
            callback(buffer.baseAddress, buffer.count, context)
        }
    }
}

final class PKRResourceFileWriter {
    private let lock = NSLock()
    private var handle: FileHandle?
    private var writeError: Error?

    init(handle: FileHandle) {
        self.handle = handle
    }

    func write(_ data: Data) {
        lock.lock()
        defer { lock.unlock() }
        guard let handle, writeError == nil else { return }
        do {
            try handle.write(contentsOf: data)
        } catch {
            writeError = error
        }
    }

    func close() -> Error? {
        lock.lock()
        defer { lock.unlock() }
        if let handle {
            do {
                try handle.close()
            } catch {
                writeError = writeError ?? error
            }
        }
        handle = nil
        return writeError
    }
}

struct PKRAssetResourceWriteResultPayload: Codable {
    var fileURL: String
    var success: Bool
    var error: PKRErrorPayload?
}

func pkrRequestAssetResource(from payload: PKRAssetResourcePayload) throws -> PHAssetResource {
    let asset = try pkrRequestAsset(localIdentifier: payload.assetLocalIdentifier)
    guard let resource = PHAssetResource.assetResources(for: asset).first(where: {
        $0.type.rawValue == payload.resourceType && $0.originalFilename == payload.originalFilename
    }) else {
        throw NSError(
            domain: "photokit-rs",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "asset resource not found: \(payload.originalFilename)"]
        )
    }
    return resource
}

func pkrBuildAssetResourceRequestOptions(_ payload: PKRAssetResourceRequestOptionsPayload) -> PHAssetResourceRequestOptions {
    let options = PHAssetResourceRequestOptions()
    options.isNetworkAccessAllowed = payload.networkAccessAllowed
    return options
}

@_cdecl("ph_asset_resource_manager_request_data")
public func ph_asset_resource_manager_request_data(
    _ resourceJSON: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ chunkCallback: @escaping PKRDataChunkCallback,
    _ sinkContext: UnsafeMutableRawPointer?,
    _ contextRetain: @escaping PKRObserverContextCallback,
    _ contextRelease: @escaping PKRObserverContextCallback,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let sinkContext else {
        pkrSetMessageError(outError, message: "missing data sink")
        return nil
    }

    do {
        let resourcePayload = try pkrDecodeJSON(resourceJSON, as: PKRAssetResourcePayload.self)
        let optionsPayload = try pkrDecodeJSON(optionsJSON, as: PKRAssetResourceRequestOptionsPayload.self)
        let resource = try pkrRequestAssetResource(from: resourcePayload)
        let manager = PHAssetResourceManager.default()
        let sink = PKRDataSink(
            callback: chunkCallback,
            context: sinkContext,
            contextRetain: contextRetain,
            contextRelease: contextRelease
        )
        let slot = PKRResultSlot<NSError?>()
        let requestID = manager.requestData(for: resource, options: pkrBuildAssetResourceRequestOptions(optionsPayload)) { data in
            sink.append(data)
        } completionHandler: { error in
            slot.fill(.success(error as NSError?))
        }

        guard let result = slot.wait(timeoutMs: timeoutMs) else {
            manager.cancelDataRequest(requestID)
            throw pkrError("asset resource request timed out")
        }

        let payload = PKRAssetResourceDataRequestPayload(
            requestID: requestID,
            error: try result.get().map(pkrErrorPayload)
        )
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}

@_cdecl("ph_asset_resource_manager_write_data_json")
public func ph_asset_resource_manager_write_data_json(
    _ resourceJSON: UnsafePointer<CChar>?,
    _ fileURL: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ timeoutMs: UInt64,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let fileURL else {
        pkrSetMessageError(outError, message: "missing file URL")
        return nil
    }

    do {
        let resourcePayload = try pkrDecodeJSON(resourceJSON, as: PKRAssetResourcePayload.self)
        let optionsPayload = try pkrDecodeJSON(optionsJSON, as: PKRAssetResourceRequestOptionsPayload.self)
        let resource = try pkrRequestAssetResource(from: resourcePayload)
        let destinationURL = try pkrFileURL(String(cString: fileURL))

        let descriptor = open(destinationURL.path, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0o644)
        guard descriptor >= 0 else {
            let code = Int(errno)
            let error = NSError(domain: NSPOSIXErrorDomain, code: code, userInfo: [NSFilePathErrorKey: destinationURL.path])
            let payload = PKRAssetResourceWriteResultPayload(
                fileURL: destinationURL.absoluteString,
                success: false,
                error: pkrErrorPayload(from: error)
            )
            return pkrCString(try pkrEncodeJSON(payload))
        }

        let writer = PKRResourceFileWriter(handle: FileHandle(fileDescriptor: descriptor, closeOnDealloc: true))
        let manager = PHAssetResourceManager.default()
        let slot = PKRResultSlot<NSError?>()
        let requestID = manager.requestData(for: resource, options: pkrBuildAssetResourceRequestOptions(optionsPayload)) { data in
            writer.write(data)
        } completionHandler: { error in
            slot.fill(.success(error as NSError?))
        }

        guard let result = slot.wait(timeoutMs: timeoutMs) else {
            manager.cancelDataRequest(requestID)
            _ = writer.close()
            try? FileManager.default.removeItem(at: destinationURL)
            throw pkrError("asset resource write timed out")
        }

        let closeError = writer.close()
        let requestError = try result.get() ?? closeError.map { $0 as NSError }
        if requestError != nil {
            try? FileManager.default.removeItem(at: destinationURL)
        }
        let payload = PKRAssetResourceWriteResultPayload(
            fileURL: destinationURL.absoluteString,
            success: requestError == nil,
            error: requestError.map(pkrErrorPayload)
        )
        return pkrCString(try pkrEncodeJSON(payload))
    } catch {
        pkrSetError(outError, error)
        return nil
    }
}
