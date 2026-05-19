import Foundation
import PhotosUI

struct PKRProjectInfoPayload: Encodable {
    var creationSource: String
    var projectType: String
    var sections: [PKRProjectSectionPayload]
    var brandingEnabled: Bool
    var pageNumbersEnabled: Bool
    var productIdentifier: String?
    var themeIdentifier: String?
}

func pkrProjectCreationSource(_ creationSource: PHProjectInfo.CreationSource) -> String {
    switch creationSource {
    case .undefined:
        return "undefined"
    case .userSelection:
        return "userSelection"
    case .album:
        return "album"
    case .memory:
        return "memory"
    case .moment:
        return "moment"
    case .project:
        return "project"
    case .projectBook:
        return "projectBook"
    case .projectCalendar:
        return "projectCalendar"
    case .projectCard:
        return "projectCard"
    case .projectPrintOrder:
        return "projectPrintOrder"
    case .projectSlideshow:
        return "projectSlideshow"
    case .projectExtension:
        return "projectExtension"
    @unknown default:
        return "undefined"
    }
}

func pkrEncodeProjectInfo(_ projectInfo: PHProjectInfo) -> PKRProjectInfoPayload {
    PKRProjectInfoPayload(
        creationSource: pkrProjectCreationSource(projectInfo.creationSource),
        projectType: projectInfo.projectType.rawValue,
        sections: projectInfo.sections.map(pkrEncodeProjectSection),
        brandingEnabled: projectInfo.brandingEnabled,
        pageNumbersEnabled: projectInfo.pageNumbersEnabled,
        productIdentifier: projectInfo.productIdentifier,
        themeIdentifier: projectInfo.themeIdentifier
    )
}

@_cdecl("ph_project_info_is_available")
public func ph_project_info_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
