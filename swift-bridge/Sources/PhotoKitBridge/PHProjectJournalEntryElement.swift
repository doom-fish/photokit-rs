import Foundation
import PhotosUI

struct PKRProjectJournalEntryElementPayload: Codable {
    var weight: Double
    var placement: PKRRectPayload?
    var date: String
    var assetElement: PKRProjectAssetElementPayload?
    var textElement: PKRProjectTextElementPayload?
}

func pkrEncodeProjectJournalEntryElement(_ element: PHProjectJournalEntryElement) -> PKRProjectJournalEntryElementPayload {
    PKRProjectJournalEntryElementPayload(
        weight: element.weight,
        placement: element.placement.isNull ? nil : pkrRectPayload(element.placement),
        date: pkrDateString(element.date) ?? element.date.description,
        assetElement: element.assetElement.map(pkrEncodeProjectAssetElement),
        textElement: element.textElement.map(pkrEncodeProjectTextElement)
    )
}

@_cdecl("ph_project_journal_entry_element_is_available")
public func ph_project_journal_entry_element_is_available() -> Int32 {
    if #available(macOS 10.13, *) {
        return PKR_OK
    }
    return PKR_ERROR
}
