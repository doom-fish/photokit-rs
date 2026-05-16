import Foundation
import Photos

struct PKRObjectChangeDetailsPayload<ObjectPayload: Codable>: Codable {
    var objectBeforeChanges: ObjectPayload
    var objectAfterChanges: ObjectPayload?
    var assetContentChanged: Bool
    var objectWasDeleted: Bool
}

func pkrEncodeObjectChangeDetails<ObjectType, ObjectPayload: Codable>(
    _ details: PHObjectChangeDetails<ObjectType>,
    transform: (ObjectType) -> ObjectPayload
) -> PKRObjectChangeDetailsPayload<ObjectPayload> {
    PKRObjectChangeDetailsPayload(
        objectBeforeChanges: transform(details.objectBeforeChanges),
        objectAfterChanges: details.objectAfterChanges.map(transform),
        assetContentChanged: details.assetContentChanged,
        objectWasDeleted: details.objectWasDeleted
    )
}
