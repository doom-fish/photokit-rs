#![allow(clippy::unsafe_derive_deserialize)]

use std::ops::Deref;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHCoordinate;
use crate::cloud_identifier::PHCloudIdentifier;
use crate::ffi;
use crate::geometry::{PHColor, PHRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectInfo.CreationSource`.
pub enum PHProjectCreationSource {
    /// Case of `PHProjectCreationSource`.
    Undefined,
    /// Case of `PHProjectCreationSource`.
    UserSelection,
    /// Case of `PHProjectCreationSource`.
    Album,
    /// Case of `PHProjectCreationSource`.
    Memory,
    /// Case of `PHProjectCreationSource`.
    Moment,
    /// Case of `PHProjectCreationSource`.
    Project,
    /// Case of `PHProjectCreationSource`.
    ProjectBook,
    /// Case of `PHProjectCreationSource`.
    ProjectCalendar,
    /// Case of `PHProjectCreationSource`.
    ProjectCard,
    /// Case of `PHProjectCreationSource`.
    ProjectPrintOrder,
    /// Case of `PHProjectCreationSource`.
    ProjectSlideshow,
    /// Case of `PHProjectCreationSource`.
    ProjectExtension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectSection.SectionType`.
pub enum PHProjectSectionType {
    /// Case of `PHProjectSectionType`.
    Undefined,
    /// Case of `PHProjectSectionType`.
    Cover,
    /// Case of `PHProjectSectionType`.
    Content,
    /// Case of `PHProjectSectionType`.
    Auxiliary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectTextElement.ElementType`.
pub enum PHProjectTextElementType {
    /// Case of `PHProjectTextElementType`.
    Body,
    /// Case of `PHProjectTextElementType`.
    Title,
    /// Case of `PHProjectTextElementType`.
    Subtitle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectElement`.
pub struct PHProjectElement {
    /// Corresponds to `PHProjectElement.weight`.
    pub weight: f64,
    /// Corresponds to `PHProjectElement.placement`.
    pub placement: Option<PHRect>,
}

impl PHProjectElement {
    /// Returns whether `PHProjectElement` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_element_is_available() == ffi::status::OK }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectRegionOfInterest`.
pub struct PHProjectRegionOfInterest {
    /// Corresponds to `PHProjectRegionOfInterest.rect`.
    pub rect: PHRect,
    /// Corresponds to `PHProjectRegionOfInterest.weight`.
    pub weight: f64,
    /// Corresponds to `PHProjectRegionOfInterest.quality`.
    pub quality: f64,
    /// Corresponds to `PHProjectRegionOfInterest.identifier`.
    pub identifier: String,
}

impl PHProjectRegionOfInterest {
    /// Returns whether `PHProjectRegionOfInterest` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_region_of_interest_is_available() == ffi::status::OK }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps map-annotation details exposed through `PHProjectMapElement`.
pub struct PHProjectMapAnnotation {
    /// Corresponds to `MKAnnotation.coordinate`.
    pub coordinate: PHCoordinate,
    /// Corresponds to `MKAnnotation.title`.
    pub title: Option<String>,
    /// Corresponds to `MKAnnotation.subtitle`.
    pub subtitle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectAssetElement`.
pub struct PHProjectAssetElement {
    #[serde(flatten)]
    /// Corresponds to the base `PHProjectElement` state.
    pub element: PHProjectElement,
    /// Corresponds to `PHProjectAssetElement.cloudAssetIdentifier`.
    pub cloud_asset_identifier: PHCloudIdentifier,
    /// Corresponds to `PHProjectAssetElement.annotation`.
    pub annotation: String,
    /// Corresponds to `PHProjectAssetElement.cropRect`.
    pub crop_rect: PHRect,
    #[serde(default)]
    /// Corresponds to `PHProjectAssetElement.regionsOfInterest`.
    pub regions_of_interest: Vec<PHProjectRegionOfInterest>,
    /// Corresponds to `PHProjectAssetElement.horizontallyFlipped`.
    pub horizontally_flipped: bool,
    /// Corresponds to `PHProjectAssetElement.verticallyFlipped`.
    pub vertically_flipped: bool,
}

impl PHProjectAssetElement {
    /// Returns whether `PHProjectAssetElement` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_asset_element_is_available() == ffi::status::OK }
    }
}

impl Deref for PHProjectAssetElement {
    type Target = PHProjectElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectTextElement`.
pub struct PHProjectTextElement {
    #[serde(flatten)]
    /// Corresponds to the base `PHProjectElement` state.
    pub element: PHProjectElement,
    /// Corresponds to `PHProjectTextElement.text`.
    pub text: String,
    /// Corresponds to `PHProjectTextElement.attributedText` serialized as RTF data.
    pub attributed_text_rtf_base64: Option<String>,
    /// Corresponds to `PHProjectTextElement.textElementType`.
    pub text_element_type: PHProjectTextElementType,
}

impl PHProjectTextElement {
    /// Returns whether `PHProjectTextElement` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_text_element_is_available() == ffi::status::OK }
    }

    /// Decodes the serialized RTF payload, if any.
    pub fn attributed_text_rtf_data(&self) -> Option<Vec<u8>> {
        self.attributed_text_rtf_base64.as_ref().and_then(|value| {
            base64::engine::general_purpose::STANDARD
                .decode(value.as_bytes())
                .ok()
        })
    }
}

impl Deref for PHProjectTextElement {
    type Target = PHProjectElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectJournalEntryElement`.
pub struct PHProjectJournalEntryElement {
    #[serde(flatten)]
    /// Corresponds to the base `PHProjectElement` state.
    pub element: PHProjectElement,
    /// Corresponds to `PHProjectJournalEntryElement.date`.
    pub date: String,
    /// Corresponds to `PHProjectJournalEntryElement.assetElement`.
    pub asset_element: Option<PHProjectAssetElement>,
    /// Corresponds to `PHProjectJournalEntryElement.textElement`.
    pub text_element: Option<PHProjectTextElement>,
}

impl PHProjectJournalEntryElement {
    /// Returns whether `PHProjectJournalEntryElement` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_journal_entry_element_is_available() == ffi::status::OK }
    }
}

impl Deref for PHProjectJournalEntryElement {
    type Target = PHProjectElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectMapElement`.
pub struct PHProjectMapElement {
    #[serde(flatten)]
    /// Corresponds to the base `PHProjectElement` state.
    pub element: PHProjectElement,
    /// Corresponds to `PHProjectMapElement.mapType`.
    pub map_type: u64,
    /// Corresponds to `PHProjectMapElement.centerCoordinate`.
    pub center_coordinate: PHCoordinate,
    /// Corresponds to `PHProjectMapElement.heading`.
    pub heading: f64,
    /// Corresponds to `PHProjectMapElement.pitch`.
    pub pitch: f64,
    /// Corresponds to `PHProjectMapElement.altitude`.
    pub altitude: f64,
    #[serde(default)]
    /// Corresponds to `PHProjectMapElement.annotations`.
    pub annotations: Vec<PHProjectMapAnnotation>,
}

impl PHProjectMapElement {
    /// Returns whether `PHProjectMapElement` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_map_element_is_available() == ffi::status::OK }
    }
}

impl Deref for PHProjectMapElement {
    type Target = PHProjectElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
/// Wraps the heterogenous `PHProjectElement` array exposed by `PHProjectSectionContent`.
pub enum PHProjectSectionElement {
    /// Case wrapping `PHProjectAssetElement`.
    Asset(PHProjectAssetElement),
    /// Case wrapping `PHProjectTextElement`.
    Text(PHProjectTextElement),
    /// Case wrapping `PHProjectJournalEntryElement`.
    JournalEntry(PHProjectJournalEntryElement),
    /// Case wrapping `PHProjectMapElement`.
    Map(PHProjectMapElement),
}

impl PHProjectSectionElement {
    /// Returns the base `PHProjectElement` shared by the variant.
    pub fn element(&self) -> &PHProjectElement {
        match self {
            Self::Asset(value) => &value.element,
            Self::Text(value) => &value.element,
            Self::JournalEntry(value) => &value.element,
            Self::Map(value) => &value.element,
        }
    }

    /// Returns the wrapped `PHProjectAssetElement`, if this is an asset entry.
    pub fn as_asset_element(&self) -> Option<&PHProjectAssetElement> {
        match self {
            Self::Asset(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the wrapped `PHProjectTextElement`, if this is a text entry.
    pub fn as_text_element(&self) -> Option<&PHProjectTextElement> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the wrapped `PHProjectJournalEntryElement`, if this is a journal entry.
    pub fn as_journal_entry_element(&self) -> Option<&PHProjectJournalEntryElement> {
        match self {
            Self::JournalEntry(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the wrapped `PHProjectMapElement`, if this is a map entry.
    pub fn as_map_element(&self) -> Option<&PHProjectMapElement> {
        match self {
            Self::Map(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectSectionContent`.
pub struct PHProjectSectionContent {
    #[serde(default)]
    /// Corresponds to `PHProjectSectionContent.elements`.
    pub elements: Vec<PHProjectSectionElement>,
    /// Corresponds to `PHProjectSectionContent.numberOfColumns`.
    pub number_of_columns: i64,
    /// Corresponds to `PHProjectSectionContent.aspectRatio`.
    pub aspect_ratio: f64,
    #[serde(default)]
    /// Corresponds to `PHProjectSectionContent.cloudAssetIdentifiers`.
    pub cloud_asset_identifiers: Vec<PHCloudIdentifier>,
    /// Corresponds to `PHProjectSectionContent.backgroundColor`.
    pub background_color: Option<PHColor>,
}

impl PHProjectSectionContent {
    /// Returns whether `PHProjectSectionContent` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_section_content_is_available() == ffi::status::OK }
    }

    /// Returns all asset elements contained in the section content.
    pub fn asset_elements(&self) -> Vec<&PHProjectAssetElement> {
        self.elements
            .iter()
            .filter_map(PHProjectSectionElement::as_asset_element)
            .collect()
    }

    /// Returns all text elements contained in the section content.
    pub fn text_elements(&self) -> Vec<&PHProjectTextElement> {
        self.elements
            .iter()
            .filter_map(PHProjectSectionElement::as_text_element)
            .collect()
    }

    /// Returns all journal-entry elements contained in the section content.
    pub fn journal_entry_elements(&self) -> Vec<&PHProjectJournalEntryElement> {
        self.elements
            .iter()
            .filter_map(PHProjectSectionElement::as_journal_entry_element)
            .collect()
    }

    /// Returns all map elements contained in the section content.
    pub fn map_elements(&self) -> Vec<&PHProjectMapElement> {
        self.elements
            .iter()
            .filter_map(PHProjectSectionElement::as_map_element)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectSection`.
pub struct PHProjectSection {
    #[serde(default)]
    /// Corresponds to `PHProjectSection.sectionContents`.
    pub section_contents: Vec<PHProjectSectionContent>,
    /// Corresponds to `PHProjectSection.sectionType`.
    pub section_type: PHProjectSectionType,
    /// Corresponds to `PHProjectSection.title`.
    pub title: String,
}

impl PHProjectSection {
    /// Returns whether `PHProjectSection` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_section_is_available() == ffi::status::OK }
    }

    /// Returns the least-curated section content suggested by PhotosUI.
    pub fn preferred_section_content(&self) -> Option<&PHProjectSectionContent> {
        self.section_contents.first()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectInfo`.
pub struct PHProjectInfo {
    /// Corresponds to `PHProjectInfo.creationSource`.
    pub creation_source: PHProjectCreationSource,
    /// Corresponds to `PHProjectInfo.projectType`.
    pub project_type: String,
    #[serde(default)]
    /// Corresponds to `PHProjectInfo.sections`.
    pub sections: Vec<PHProjectSection>,
    /// Corresponds to `PHProjectInfo.brandingEnabled`.
    pub branding_enabled: bool,
    /// Corresponds to `PHProjectInfo.pageNumbersEnabled`.
    pub page_numbers_enabled: bool,
    /// Corresponds to `PHProjectInfo.productIdentifier`.
    pub product_identifier: Option<String>,
    /// Corresponds to `PHProjectInfo.themeIdentifier`.
    pub theme_identifier: Option<String>,
}

impl PHProjectInfo {
    /// Returns whether `PHProjectInfo` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_info_is_available() == ffi::status::OK }
    }

    /// Returns all cloud asset identifiers referenced anywhere in the project info.
    pub fn all_cloud_asset_identifiers(&self) -> Vec<PHCloudIdentifier> {
        self.sections
            .iter()
            .flat_map(|section| {
                section
                    .section_contents
                    .iter()
                    .flat_map(|content| content.cloud_asset_identifiers.clone())
            })
            .collect()
    }
}
