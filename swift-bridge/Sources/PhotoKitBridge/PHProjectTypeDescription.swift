import AppKit
import Foundation
import PhotosUI

struct PKRProjectTypeDescriptionPayload: Encodable {
    var projectType: String
    var localizedTitle: String
    var localizedDescription: String?
    var localizedAttributedDescriptionRtfBase64: String?
    var imageTiffBase64: String?
    var subtypeDescriptions: [PKRProjectTypeDescriptionPayload]
    var canProvideSubtypes: Bool
}

func pkrEncodeProjectTypeDescription(_ description: PHProjectTypeDescription) -> PKRProjectTypeDescriptionPayload {
    let attributedDescriptionData = description.localizedAttributedDescription.flatMap { attributedDescription -> String? in
        let range = NSRange(location: 0, length: attributedDescription.length)
        let data = try? attributedDescription.data(
            from: range,
            documentAttributes: [.documentType: NSAttributedString.DocumentType.rtf]
        )
        return data?.base64EncodedString()
    }
    return PKRProjectTypeDescriptionPayload(
        projectType: description.projectType.rawValue,
        localizedTitle: description.localizedTitle,
        localizedDescription: description.localizedDescription,
        localizedAttributedDescriptionRtfBase64: attributedDescriptionData,
        imageTiffBase64: description.image?.tiffRepresentation?.base64EncodedString(),
        subtypeDescriptions: description.subtypeDescriptions.map(pkrEncodeProjectTypeDescription),
        canProvideSubtypes: description.canProvideSubtypes
    )
}

@_cdecl("ph_project_type_description_is_available")
public func ph_project_type_description_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
