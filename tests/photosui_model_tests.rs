use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use photokit::prelude::*;

fn sample_region_of_interest() -> PHProjectRegionOfInterest {
    PHProjectRegionOfInterest {
        rect: PHRect::new(0.1, 0.2, 0.3, 0.4),
        weight: 0.9,
        quality: 0.8,
        identifier: "person-1".to_owned(),
    }
}

fn sample_asset_element(regions_of_interest: Vec<PHProjectRegionOfInterest>) -> PHProjectAssetElement {
    PHProjectAssetElement {
        element: PHProjectElement {
            weight: 1.0,
            placement: Some(PHRect::new(0.0, 0.0, 1.0, 1.0)),
        },
        cloud_asset_identifier: PHCloudIdentifier::new("cloud-1"),
        annotation: "Cover photo".to_owned(),
        crop_rect: PHRect::new(0.0, 0.0, 1.0, 1.0),
        regions_of_interest,
        horizontally_flipped: false,
        vertically_flipped: false,
    }
}

fn sample_text_element() -> PHProjectTextElement {
    PHProjectTextElement {
        element: PHProjectElement {
            weight: 0.5,
            placement: Some(PHRect::new(0.0, 0.0, 0.5, 0.2)),
        },
        text: "Trip".to_owned(),
        attributed_text_rtf_base64: Some("e1xydGYxXGFuc2k=".to_owned()),
        text_element_type: PHProjectTextElementType::Title,
    }
}

fn sample_journal_entry(
    asset_element: PHProjectAssetElement,
    text_element: PHProjectTextElement,
) -> PHProjectJournalEntryElement {
    PHProjectJournalEntryElement {
        element: PHProjectElement {
            weight: 0.4,
            placement: None,
        },
        date: "2026-05-18T00:00:00Z".to_owned(),
        asset_element: Some(asset_element),
        text_element: Some(text_element),
    }
}

fn sample_map_element() -> PHProjectMapElement {
    PHProjectMapElement {
        element: PHProjectElement {
            weight: 0.2,
            placement: None,
        },
        map_type: 0,
        center_coordinate: PHCoordinate {
            latitude: 59.3293,
            longitude: 18.0686,
        },
        heading: 0.0,
        pitch: 0.0,
        altitude: 100.0,
        annotations: vec![PHProjectMapAnnotation {
            coordinate: PHCoordinate {
                latitude: 59.3293,
                longitude: 18.0686,
            },
            title: Some("Start".to_owned()),
            subtitle: None,
        }],
    }
}

#[test]
fn photosui_type_availability() {
    assert!(PHPickerConfiguration::is_available());
    assert!(PHPickerFilter::is_available());
    assert!(PHPickerResult::is_available());
    assert!(PHPickerViewController::is_available());
    assert!(PHPickerViewControllerDelegate::is_available());
    assert!(PHLivePhotoView::is_available());
    assert!(PHLivePhotoViewDelegate::is_available());
    assert!(PHContentEditingController::is_available());
    assert!(PHProjectInfo::is_available());
    assert!(PHProjectElement::is_available());
    assert!(PHProjectAssetElement::is_available());
    assert!(PHProjectRegionOfInterest::is_available());
    assert!(PHProjectSection::is_available());
    assert!(PHProjectSectionContent::is_available());
    assert!(PHProjectTextElement::is_available());
    assert!(PHProjectJournalEntryElement::is_available());
    assert!(PHProjectMapElement::is_available());
    assert!(PHProjectTypeDescription::is_available());
    assert!(PHProjectTypeDescriptionDataSource::is_available());
    assert!(PHProjectTypeDescriptionInvalidator::is_available());
    assert!(PHProjectExtensionContext::is_available());
    assert!(PHProjectExtensionController::is_available());
}

#[test]
fn photosui_picker_helpers_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let filter = PHPickerFilter::any_matching(vec![
        PHPickerFilter::images(),
        PHPickerFilter::live_photos(),
    ]);
    let description = filter.description()?;
    assert!(!description.is_empty());

    let picker_config = PHPickerConfiguration::new()
        .set_selection_limit(2)
        .set_selection(PHPickerConfigurationSelection::Ordered)
        .set_filter(filter)
        .set_edges_without_content_margins(PHDirectionalRectEdge::TOP | PHDirectionalRectEdge::BOTTOM)
        .set_disabled_capabilities(PHPickerCapabilities::SEARCH | PHPickerCapabilities::SELECTION_ACTIONS);
    assert_eq!(picker_config.selection_limit, 2);
    assert!(picker_config
        .edges_without_content_margins
        .is_some_and(|edges| edges.contains(PHDirectionalRectEdge::TOP)));

    let update_configuration = PHPickerUpdateConfiguration::new()
        .set_selection_limit(3)
        .set_edges_without_content_margins(PHDirectionalRectEdge::ALL);
    assert_eq!(update_configuration.selection_limit, Some(3));

    Ok(())
}

#[test]
fn photosui_project_models_round_trip() {
    let asset_element = sample_asset_element(vec![sample_region_of_interest()]);
    let cloud_asset_identifier = asset_element.cloud_asset_identifier.clone();
    let text_element = sample_text_element();
    let journal_entry = sample_journal_entry(asset_element.clone(), text_element.clone());
    let map_element = sample_map_element();
    assert!(text_element.attributed_text_rtf_data().is_some());

    let section_content = PHProjectSectionContent {
        elements: vec![
            PHProjectSectionElement::Asset(asset_element),
            PHProjectSectionElement::Text(text_element),
            PHProjectSectionElement::JournalEntry(journal_entry),
            PHProjectSectionElement::Map(map_element),
        ],
        number_of_columns: 1,
        aspect_ratio: 1.5,
        cloud_asset_identifiers: vec![cloud_asset_identifier],
        background_color: Some(PHColor::new(0.1, 0.2, 0.3, 1.0)),
    };
    assert_eq!(section_content.asset_elements().len(), 1);
    assert_eq!(section_content.text_elements().len(), 1);
    assert_eq!(section_content.journal_entry_elements().len(), 1);
    assert_eq!(section_content.map_elements().len(), 1);
    assert!(matches!(
        section_content.elements[0].as_asset_element(),
        Some(value) if value.annotation == "Cover photo"
    ));

    let section = PHProjectSection {
        section_contents: vec![section_content],
        section_type: PHProjectSectionType::Content,
        title: "Day 1".to_owned(),
    };
    assert!(section.preferred_section_content().is_some());

    let project_info = PHProjectInfo {
        creation_source: PHProjectCreationSource::Memory,
        project_type: "book".to_owned(),
        sections: vec![section],
        branding_enabled: true,
        page_numbers_enabled: false,
        product_identifier: Some("product-1".to_owned()),
        theme_identifier: Some("theme-1".to_owned()),
    };
    assert_eq!(project_info.all_cloud_asset_identifiers().len(), 1);
}

#[test]
fn photosui_project_extension_helpers_round_trip() {
    let type_description = PHProjectTypeDescription::new("book", "Book")
        .set_localized_description("Hardcover")
        .set_localized_attributed_description_rtf_base64("e1xydGYxXGFuc2k=")
        .set_image_tiff_base64("aGVsbG8=")
        .set_subtype_descriptions(vec![PHProjectTypeDescription::new("book.small", "Small")])
        .set_can_provide_subtypes(true);
    assert_eq!(type_description.image_tiff_data(), Some(b"hello".to_vec()));
    assert!(type_description.localized_attributed_description_rtf_data().is_some());

    let footer_calls = Arc::new(AtomicUsize::new(0));
    let footer_calls_clone = Arc::clone(&footer_calls);
    let data_source = PHProjectTypeDescriptionDataSource::new(
        move |_| vec![PHProjectTypeDescription::new("book.small", "Small")],
        |_| Some(PHProjectTypeDescription::new("book", "Book")),
        move |_| {
            footer_calls_clone.fetch_add(1, Ordering::Relaxed);
            Some("Footer".to_owned())
        },
    );
    assert_eq!(data_source.subtypes_for_project_type("book").len(), 1);
    assert!(data_source.type_description_for_project_type("book").is_some());
    assert_eq!(
        data_source.footer_text_for_subtypes_of_project_type("book"),
        Some("Footer".to_owned())
    );
    assert_eq!(footer_calls.load(Ordering::Relaxed), 1);

    let invalidate_calls = Arc::new(AtomicUsize::new(0));
    let invalidate_calls_clone = Arc::clone(&invalidate_calls);
    let invalidator = PHProjectTypeDescriptionInvalidator::new(
        move |_| {
            invalidate_calls_clone.fetch_add(1, Ordering::Relaxed);
        },
        {
            let invalidate_calls = Arc::clone(&invalidate_calls);
            move |_| {
                invalidate_calls.fetch_add(1, Ordering::Relaxed);
            }
        },
    );
    invalidator.invalidate_type_description_for_project_type("book");
    invalidator.invalidate_footer_text_for_subtypes_of_project_type("book");
    assert_eq!(invalidate_calls.load(Ordering::Relaxed), 2);

    let project_controller = PHProjectExtensionController::new(|_, _| Ok(()), |_| Ok(()), || {})
        .with_supported_project_types(vec![type_description])
        .with_type_description_data_source(PHProjectTypeDescriptionDataSource::new(
            |_| vec![],
            |_| None,
            |_| None,
        ));
    assert_eq!(project_controller.supported_project_types().len(), 1);
    assert!(project_controller.type_description_data_source().is_some());
}

#[test]
fn photosui_content_editing_helpers_round_trip() {
    let cancel_calls = Arc::new(AtomicUsize::new(0));
    let cancel_calls_clone = Arc::clone(&cancel_calls);
    let content_controller = PHContentEditingController::new(
        |data| data.format_identifier == "fmt",
        |_, _| {},
        || Ok(None),
        move || {
            cancel_calls_clone.fetch_add(1, Ordering::Relaxed);
        },
        || true,
    );
    let adjustment_data = PHAdjustmentData {
        format_identifier: "fmt".to_owned(),
        format_version: "1".to_owned(),
        data_base64: "aGVsbG8=".to_owned(),
    };
    assert!(content_controller.can_handle_adjustment_data(&adjustment_data));
    assert!(content_controller.should_show_cancel_confirmation());
    content_controller.cancel_content_editing();
    assert_eq!(cancel_calls.load(Ordering::Relaxed), 1);

    let placeholder = PHContentEditingPlaceholderImage {
        tiff_data_base64: "aGVsbG8=".to_owned(),
        width: 10.0,
        height: 20.0,
    };
    assert_eq!(placeholder.tiff_data(), b"hello".to_vec());
}
