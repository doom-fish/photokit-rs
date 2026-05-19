import AppKit
import Foundation
import PhotosUI

struct PKRProjectTextElementPayload: Codable {
    var weight: Double
    var placement: PKRRectPayload?
    var text: String
    var attributedTextRtfBase64: String?
    var textElementType: String
}

func pkrProjectTextElementType(_ textElementType: PHProjectTextElement.ElementType) -> String {
    switch textElementType {
    case .body:
        return "body"
    case .title:
        return "title"
    case .subtitle:
        return "subtitle"
    @unknown default:
        return "body"
    }
}

func pkrRTFBase64(_ attributedString: NSAttributedString?) -> String? {
    guard let attributedString else { return nil }
    let range = NSRange(location: 0, length: attributedString.length)
    let data = try? attributedString.data(
        from: range,
        documentAttributes: [.documentType: NSAttributedString.DocumentType.rtf]
    )
    return data?.base64EncodedString()
}

func pkrEncodeProjectTextElement(_ element: PHProjectTextElement) -> PKRProjectTextElementPayload {
    PKRProjectTextElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement),
        text: element.text,
        attributedTextRtfBase64: pkrRTFBase64(element.attributedText),
        textElementType: pkrProjectTextElementType(element.textElementType)
    )
}

@_cdecl("ph_project_text_element_is_available")
public func ph_project_text_element_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
