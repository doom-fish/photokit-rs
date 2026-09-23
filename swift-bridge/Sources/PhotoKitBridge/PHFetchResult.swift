import Foundation
import Photos
import PhotoKitObjCBridge

func pkrCheckedFetchResult<ObjectType>(_ result: PHFetchResult<ObjectType>) throws -> PHFetchResult<ObjectType> {
    var error: NSError?
    guard PKRFetchResultCount(result, &error) >= 0 else {
        throw error ?? pkrError("fetch failed")
    }
    return result
}

func pkrCollectFetchResult<ObjectType, Payload>(
    _ result: PHFetchResult<ObjectType>,
    transform: @escaping (ObjectType) -> Payload
) throws -> [Payload] {
    var payloads: [Payload] = []
    try pkrCheckedFetchResult(result).enumerateObjects { object, _, _ in
        payloads.append(transform(object))
    }
    return payloads
}

func pkrIndexArray(_ indexSet: NSIndexSet?) -> [UInt64]? {
    guard let indexSet else { return nil }
    return (indexSet as IndexSet).map(UInt64.init)
}
