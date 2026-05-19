import Foundation
import PhotosUI

enum PKRProjectSectionElementPayload: Encodable {
    case asset(PKRProjectAssetElementPayload)
    case text(PKRProjectTextElementPayload)
    case journalEntry(PKRProjectJournalEntryElementPayload)
    case map(PKRProjectMapElementPayload)

    enum CodingKeys: String, CodingKey {
        case kind
        case payload
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case let .asset(payload):
            try container.encode("asset", forKey: .kind)
            try container.encode(payload, forKey: .payload)
        case let .text(payload):
            try container.encode("text", forKey: .kind)
            try container.encode(payload, forKey: .payload)
        case let .journalEntry(payload):
            try container.encode("journalEntry", forKey: .kind)
            try container.encode(payload, forKey: .payload)
        case let .map(payload):
            try container.encode("map", forKey: .kind)
            try container.encode(payload, forKey: .payload)
        }
    }
}

struct PKRProjectSectionContentPayload: Encodable {
    var elements: [PKRProjectSectionElementPayload]
    var numberOfColumns: Int
    var aspectRatio: Double
    var cloudAssetIdentifiers: [PKRCloudIdentifierPayload]
    var backgroundColor: PKRColorPayload?
}

func pkrEncodeProjectSectionElement(_ element: PHProjectElement) -> PKRProjectSectionElementPayload {
    if let assetElement = element as? PHProjectAssetElement {
        return .asset(pkrEncodeProjectAssetElement(assetElement))
    }
    if let textElement = element as? PHProjectTextElement {
        return .text(pkrEncodeProjectTextElement(textElement))
    }
    if let journalEntryElement = element as? PHProjectJournalEntryElement {
        return .journalEntry(pkrEncodeProjectJournalEntryElement(journalEntryElement))
    }
    if #available(macOS 10.14, *), let mapElement = element as? PHProjectMapElement {
        return .map(pkrEncodeProjectMapElement(mapElement))
    }
    return .text(PKRProjectTextElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement),
        text: "",
        attributedTextRtfBase64: nil,
        textElementType: "body"
    ))
}

func pkrEncodeProjectSectionContent(_ content: PHProjectSectionContent) -> PKRProjectSectionContentPayload {
    PKRProjectSectionContentPayload(
        elements: content.elements.map(pkrEncodeProjectSectionElement),
        numberOfColumns: content.numberOfColumns,
        aspectRatio: content.aspectRatio,
        cloudAssetIdentifiers: content.cloudAssetIdentifiers.compactMap(pkrEncodeCloudIdentifier),
        backgroundColor: pkrColorPayload(content.backgroundColor)
    )
}

@_cdecl("ph_project_section_content_is_available")
public func ph_project_section_content_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
