#![allow(clippy::unsafe_derive_deserialize)]

use core::ffi::c_void;
use std::ptr::{self, NonNull};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::photo_library::PHPhotoLibrary;
use crate::private::{cstring_from_str, parse_json_ptr};
use crate::project::PHProject;
use crate::project_info::PHProjectInfo;

type ProjectSubtypesCallback = dyn Fn(&str) -> Vec<PHProjectTypeDescription> + Send + Sync;
type ProjectTypeDescriptionCallback =
    dyn Fn(&str) -> Option<PHProjectTypeDescription> + Send + Sync;
type ProjectFooterCallback = dyn Fn(&str) -> Option<String> + Send + Sync;
type InvalidateTypeDescriptionCallback = dyn Fn(&str) + Send + Sync;
type InvalidateFooterCallback = dyn Fn(&str) + Send + Sync;
type BeginProjectCallback =
    dyn Fn(&PHProjectExtensionContext, &PHProjectInfo) -> Result<(), PhotoKitError> + Send + Sync;
type ResumeProjectCallback =
    dyn Fn(&PHProjectExtensionContext) -> Result<(), PhotoKitError> + Send + Sync;
type FinishProjectCallback = dyn Fn() + Send + Sync;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHProjectTypeDescription`.
pub struct PHProjectTypeDescription {
    /// Corresponds to `PHProjectTypeDescription.projectType`.
    pub project_type: String,
    /// Corresponds to `PHProjectTypeDescription.localizedTitle`.
    pub localized_title: String,
    /// Corresponds to `PHProjectTypeDescription.localizedDescription`.
    pub localized_description: Option<String>,
    /// Corresponds to `PHProjectTypeDescription.localizedAttributedDescription` serialized as RTF data.
    pub localized_attributed_description_rtf_base64: Option<String>,
    /// Corresponds to `PHProjectTypeDescription.image` serialized as TIFF data.
    pub image_tiff_base64: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHProjectTypeDescription.subtypeDescriptions`.
    pub subtype_descriptions: Vec<Self>,
    /// Corresponds to `PHProjectTypeDescription.canProvideSubtypes`.
    pub can_provide_subtypes: bool,
}

impl PHProjectTypeDescription {
    /// Creates a project-type description.
    pub fn new(project_type: impl Into<String>, localized_title: impl Into<String>) -> Self {
        Self {
            project_type: project_type.into(),
            localized_title: localized_title.into(),
            ..Self::default()
        }
    }

    /// Returns whether `PHProjectTypeDescription` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_type_description_is_available() == ffi::status::OK }
    }

    /// Updates `localizedDescription`.
    pub fn set_localized_description(
        mut self,
        localized_description: impl Into<String>,
    ) -> Self {
        self.localized_description = Some(localized_description.into());
        self
    }

    /// Updates the serialized attributed-description payload.
    pub fn set_localized_attributed_description_rtf_base64(
        mut self,
        localized_attributed_description_rtf_base64: impl Into<String>,
    ) -> Self {
        self.localized_attributed_description_rtf_base64 =
            Some(localized_attributed_description_rtf_base64.into());
        self
    }

    /// Updates the serialized image payload.
    pub fn set_image_tiff_base64(mut self, image_tiff_base64: impl Into<String>) -> Self {
        self.image_tiff_base64 = Some(image_tiff_base64.into());
        self
    }

    /// Updates `subtypeDescriptions`.
    pub fn set_subtype_descriptions(
        mut self,
        subtype_descriptions: Vec<Self>,
    ) -> Self {
        self.subtype_descriptions = subtype_descriptions;
        self
    }

    /// Updates `canProvideSubtypes`.
    pub fn set_can_provide_subtypes(mut self, can_provide_subtypes: bool) -> Self {
        self.can_provide_subtypes = can_provide_subtypes;
        self
    }

    /// Decodes the serialized attributed-description payload, if any.
    pub fn localized_attributed_description_rtf_data(&self) -> Option<Vec<u8>> {
        self.localized_attributed_description_rtf_base64
            .as_ref()
            .and_then(|value| {
                base64::engine::general_purpose::STANDARD
                    .decode(value.as_bytes())
                    .ok()
            })
    }

    /// Decodes the serialized image payload, if any.
    pub fn image_tiff_data(&self) -> Option<Vec<u8>> {
        self.image_tiff_base64.as_ref().and_then(|value| {
            base64::engine::general_purpose::STANDARD
                .decode(value.as_bytes())
                .ok()
        })
    }
}

/// Rust-side wrapper matching the `PHProjectTypeDescriptionDataSource` protocol contract.
pub struct PHProjectTypeDescriptionDataSource {
    subtypes: Box<ProjectSubtypesCallback>,
    type_description: Box<ProjectTypeDescriptionCallback>,
    footer: Box<ProjectFooterCallback>,
}

impl PHProjectTypeDescriptionDataSource {
    /// Creates a data-source helper for project-type descriptions.
    pub fn new<Subtypes, TypeDescription, Footer>(
        subtypes_callback: Subtypes,
        type_description_callback: TypeDescription,
        footer_callback: Footer,
    ) -> Self
    where
        Subtypes: Fn(&str) -> Vec<PHProjectTypeDescription> + Send + Sync + 'static,
        TypeDescription: Fn(&str) -> Option<PHProjectTypeDescription> + Send + Sync + 'static,
        Footer: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            subtypes: Box::new(subtypes_callback),
            type_description: Box::new(type_description_callback),
            footer: Box::new(footer_callback),
        }
    }

    /// Returns whether `PHProjectTypeDescriptionDataSource` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_type_description_data_source_is_available() == ffi::status::OK }
    }

    /// Wraps `subtypesForProjectType(_:)`.
    pub fn subtypes_for_project_type(&self, project_type: &str) -> Vec<PHProjectTypeDescription> {
        (self.subtypes)(project_type)
    }

    /// Wraps `typeDescriptionForProjectType(_:)`.
    pub fn type_description_for_project_type(
        &self,
        project_type: &str,
    ) -> Option<PHProjectTypeDescription> {
        (self.type_description)(project_type)
    }

    /// Wraps `footerTextForSubtypesOfProjectType(_:)`.
    pub fn footer_text_for_subtypes_of_project_type(&self, project_type: &str) -> Option<String> {
        (self.footer)(project_type)
    }
}

impl core::fmt::Debug for PHProjectTypeDescriptionDataSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHProjectTypeDescriptionDataSource")
            .finish_non_exhaustive()
    }
}

/// Rust-side wrapper matching the `PHProjectTypeDescriptionInvalidator` protocol contract.
pub struct PHProjectTypeDescriptionInvalidator {
    invalidate_type_description: Box<InvalidateTypeDescriptionCallback>,
    invalidate_footer_text: Box<InvalidateFooterCallback>,
}

impl PHProjectTypeDescriptionInvalidator {
    /// Creates an invalidator helper.
    pub fn new<TypeDescription, Footer>(
        invalidate_type_description_callback: TypeDescription,
        invalidate_footer_callback: Footer,
    ) -> Self
    where
        TypeDescription: Fn(&str) + Send + Sync + 'static,
        Footer: Fn(&str) + Send + Sync + 'static,
    {
        Self {
            invalidate_type_description: Box::new(invalidate_type_description_callback),
            invalidate_footer_text: Box::new(invalidate_footer_callback),
        }
    }

    /// Creates a no-op invalidator helper.
    pub fn noop() -> Self {
        Self::new(|_| {}, |_| {})
    }

    /// Returns whether `PHProjectTypeDescriptionInvalidator` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_type_description_invalidator_is_available() == ffi::status::OK }
    }

    /// Wraps `invalidateTypeDescriptionForProjectType(_:)`.
    pub fn invalidate_type_description_for_project_type(&self, project_type: &str) {
        (self.invalidate_type_description)(project_type);
    }

    /// Wraps `invalidateFooterTextForSubtypesOfProjectType(_:)`.
    pub fn invalidate_footer_text_for_subtypes_of_project_type(&self, project_type: &str) {
        (self.invalidate_footer_text)(project_type);
    }
}

impl core::fmt::Debug for PHProjectTypeDescriptionInvalidator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHProjectTypeDescriptionInvalidator")
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
/// Wraps `PHProjectExtensionContext`.
pub struct PHProjectExtensionContext {
    raw: NonNull<c_void>,
}

impl PHProjectExtensionContext {
    /// Returns whether `PHProjectExtensionContext` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_extension_context_is_available() == ffi::status::OK }
    }

    /// Creates a wrapper from a retained `PHProjectExtensionContext` pointer.
    ///
    /// # Safety
    ///
    /// `raw` must be a valid retained `PHProjectExtensionContext` reference obtained
    /// from the Swift bridge, and the returned wrapper must be the unique owner
    /// responsible for releasing it.
    pub unsafe fn from_raw(raw: *mut c_void) -> Result<Self, PhotoKitError> {
        NonNull::new(raw)
            .map(|raw| Self { raw })
            .ok_or_else(|| {
                PhotoKitError::OperationFailed(
                    "failed to create PHProjectExtensionContext handle".to_owned(),
                )
            })
    }

    /// Returns the `PHPhotoLibrary` exposed by the extension context.
    pub fn photo_library(&self) -> Result<PHPhotoLibrary, PhotoKitError> {
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_project_extension_context_photo_library(self.raw.as_ptr(), &mut error)
        };
        let raw = NonNull::new(raw).ok_or_else(|| unsafe {
            PhotoKitError::from_error_ptr(error, "project extension photo library lookup failed")
        })?;
        Ok(PHPhotoLibrary { raw })
    }

    /// Returns the underlying `PHProject` as a serialized snapshot.
    pub fn project(&self) -> Result<PHProject, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_project_extension_context_project_json(self.raw.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "project extension project lookup failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHProject") }
        }
    }

    /// Wraps `showEditorForAsset(_:)`.
    pub fn show_editor_for_asset(&self, asset: &PHAsset) -> Result<(), PhotoKitError> {
        let asset_identifier =
            cstring_from_str(&asset.local_identifier, "project extension asset identifier")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_project_extension_context_show_editor_for_asset(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "show project asset editor failed") })
        }
    }

    /// Wraps `updatedProjectInfoFromProjectInfo(_:completion:)` using the current project contents.
    pub fn updated_project_info_from_current_project(
        &self,
        timeout_ms: u64,
    ) -> Result<Option<PHProjectInfo>, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_project_extension_context_updated_project_info_json(
                self.raw.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "updated project info request failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHProjectInfo option") }
        }
    }
}

impl Drop for PHProjectExtensionContext {
    fn drop(&mut self) {
        unsafe { ffi::ph_project_extension_context_release(self.raw.as_ptr()) };
    }
}

/// Rust-side wrapper matching the `PHProjectExtensionController` protocol contract.
pub struct PHProjectExtensionController {
    supported_project_types: Vec<PHProjectTypeDescription>,
    type_description_data_source: Option<PHProjectTypeDescriptionDataSource>,
    begin_callback: Box<BeginProjectCallback>,
    resume_callback: Box<ResumeProjectCallback>,
    finish_callback: Box<FinishProjectCallback>,
}

impl PHProjectExtensionController {
    /// Creates a project-extension controller helper.
    pub fn new<Begin, Resume, Finish>(
        begin_callback: Begin,
        resume_callback: Resume,
        finish_callback: Finish,
    ) -> Self
    where
        Begin: Fn(&PHProjectExtensionContext, &PHProjectInfo) -> Result<(), PhotoKitError>
            + Send
            + Sync
            + 'static,
        Resume: Fn(&PHProjectExtensionContext) -> Result<(), PhotoKitError>
            + Send
            + Sync
            + 'static,
        Finish: Fn() + Send + Sync + 'static,
    {
        Self {
            supported_project_types: Vec::new(),
            type_description_data_source: None,
            begin_callback: Box::new(begin_callback),
            resume_callback: Box::new(resume_callback),
            finish_callback: Box::new(finish_callback),
        }
    }

    /// Returns whether `PHProjectExtensionController` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_project_extension_controller_is_available() == ffi::status::OK }
    }

    /// Sets the deprecated `supportedProjectTypes` list.
    pub fn with_supported_project_types(
        mut self,
        supported_project_types: Vec<PHProjectTypeDescription>,
    ) -> Self {
        self.supported_project_types = supported_project_types;
        self
    }

    /// Sets the data source used by `typeDescriptionDataSourceForCategory:invalidator:`.
    pub fn with_type_description_data_source(
        mut self,
        type_description_data_source: PHProjectTypeDescriptionDataSource,
    ) -> Self {
        self.type_description_data_source = Some(type_description_data_source);
        self
    }

    /// Returns the configured `supportedProjectTypes` list.
    pub fn supported_project_types(&self) -> &[PHProjectTypeDescription] {
        &self.supported_project_types
    }

    /// Returns the configured type-description data source, if any.
    pub fn type_description_data_source(&self) -> Option<&PHProjectTypeDescriptionDataSource> {
        self.type_description_data_source.as_ref()
    }

    /// Wraps `beginProjectWithExtensionContext:projectInfo:completion:`.
    pub fn begin_project_with_extension_context(
        &self,
        extension_context: &PHProjectExtensionContext,
        project_info: &PHProjectInfo,
    ) -> Result<(), PhotoKitError> {
        (self.begin_callback)(extension_context, project_info)
    }

    /// Wraps `resumeProjectWithExtensionContext:completion:`.
    pub fn resume_project_with_extension_context(
        &self,
        extension_context: &PHProjectExtensionContext,
    ) -> Result<(), PhotoKitError> {
        (self.resume_callback)(extension_context)
    }

    /// Wraps `finishProjectWithCompletionHandler:`.
    pub fn finish_project_with_completion_handler(&self) {
        (self.finish_callback)();
    }
}

impl core::fmt::Debug for PHProjectExtensionController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHProjectExtensionController")
            .field("supported_project_types", &self.supported_project_types)
            .field(
                "type_description_data_source",
                &self.type_description_data_source.as_ref().map(|_| "configured"),
            )
            .finish_non_exhaustive()
    }
}
