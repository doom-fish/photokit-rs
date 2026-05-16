import Foundation
import Photos

func pkrCollectFetchResult<ObjectType, Payload>(
    _ result: PHFetchResult<ObjectType>,
    transform: @escaping (ObjectType) -> Payload
) -> [Payload] {
    var payloads: [Payload] = []
    result.enumerateObjects { object, _, _ in
        payloads.append(transform(object))
    }
    return payloads
}

func pkrIndexArray(_ indexSet: NSIndexSet?) -> [UInt64]? {
    guard let indexSet else { return nil }
    return (indexSet as IndexSet).map(UInt64.init)
}
