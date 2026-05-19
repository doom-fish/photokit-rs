import Foundation
import PhotosUI

struct PKRProjectSectionPayload: Encodable {
    var sectionContents: [PKRProjectSectionContentPayload]
    var sectionType: String
    var title: String
}

func pkrProjectSectionType(_ sectionType: PHProjectSection.SectionType) -> String {
    switch sectionType {
    case .undefined:
        return "undefined"
    case .cover:
        return "cover"
    case .content:
        return "content"
    case .auxiliary:
        return "auxiliary"
    @unknown default:
        return "undefined"
    }
}

func pkrEncodeProjectSection(_ section: PHProjectSection) -> PKRProjectSectionPayload {
    PKRProjectSectionPayload(
        sectionContents: section.sectionContents.map(pkrEncodeProjectSectionContent),
        sectionType: pkrProjectSectionType(section.sectionType),
        title: section.title
    )
}

@_cdecl("ph_project_section_is_available")
public func ph_project_section_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
