#![allow(clippy::unsafe_derive_deserialize)]

use core::ffi::{c_char, c_void};
use std::ops::{BitOr, BitOrAssign};
use std::ptr::{self, NonNull};

use doom_fish_utils::panic_safe::catch_user_panic;
use serde::{Deserialize, Serialize};

use crate::asset::PHAssetPlaybackStyle;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::photo_library::PHPhotoLibrary;
use crate::private::{cstring_from_str, json_cstring, take_string};

macro_rules! option_set_type {
    ($name:ident, $raw:ty, { $($constant:ident = $value:expr,)* }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
        #[serde(transparent)]
        #[doc = concat!("Wraps `", stringify!($name), "`.")]
        pub struct $name(
            #[doc = concat!("Raw value for `", stringify!($name), "`.")]
            pub $raw,
        );

        impl $name {
            $(#[doc = concat!("Constant on `", stringify!($name), "`.")] pub const $constant: Self = Self($value);)*

            #[doc = concat!("Returns the raw bitmask for `", stringify!($name), "`.")]
            pub const fn bits(self) -> $raw {
                self.0
            }

            #[doc = concat!("Returns whether this `", stringify!($name), "` contains `other`.")]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl From<$raw> for $name {
            fn from(value: $raw) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $raw {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
    };
}

option_set_type!(PHDirectionalRectEdge, u64, {
    NONE = 0,
    TOP = 1,
    LEADING = 1 << 1,
    BOTTOM = 1 << 2,
    TRAILING = 1 << 3,
    ALL = (1 << 4) - 1,
});

option_set_type!(PHPickerCapabilities, u64, {
    NONE = 0,
    SEARCH = 1 << 0,
    STAGING_AREA = 1 << 1,
    COLLECTION_NAVIGATION = 1 << 2,
    SELECTION_ACTIONS = 1 << 3,
    SENSITIVITY_ANALYSIS_INTERVENTION = 1 << 4,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerConfigurationAssetRepresentationMode`.
pub enum PHPickerConfigurationAssetRepresentationMode {
    #[default]
    /// Case of `PHPickerConfigurationAssetRepresentationMode`.
    Automatic,
    /// Case of `PHPickerConfigurationAssetRepresentationMode`.
    Current,
    /// Case of `PHPickerConfigurationAssetRepresentationMode`.
    Compatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerConfigurationSelection`.
pub enum PHPickerConfigurationSelection {
    #[default]
    /// Case of `PHPickerConfigurationSelection`.
    Default,
    /// Case of `PHPickerConfigurationSelection`.
    Ordered,
    /// Case of `PHPickerConfigurationSelection`.
    Continuous,
    /// Case of `PHPickerConfigurationSelection`.
    ContinuousAndOrdered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerMode`.
pub enum PHPickerMode {
    #[default]
    /// Case of `PHPickerMode`.
    Default,
    /// Case of `PHPickerMode`.
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// Wraps `PHPickerFilter`.
pub enum PHPickerFilter {
    /// Filter matching images.
    Images,
    /// Filter matching videos.
    Videos,
    /// Filter matching live photos.
    LivePhotos,
    /// Filter matching depth-effect photos.
    DepthEffectPhotos,
    /// Filter matching burst assets.
    Bursts,
    /// Filter matching panorama assets.
    Panoramas,
    /// Filter matching screenshots.
    Screenshots,
    /// Filter matching screen recordings.
    ScreenRecordings,
    /// Filter matching cinematic videos.
    CinematicVideos,
    /// Filter matching slow-motion videos.
    SlomoVideos,
    /// Filter matching time-lapse videos.
    TimelapseVideos,
    /// Filter matching spatial media.
    SpatialMedia,
    /// Filter matching a playback style.
    PlaybackStyle {
        /// Corresponds to `PHPickerFilter.playbackStyleFilter(_:)`.
        playback_style: PHAssetPlaybackStyle,
    },
    /// Filter matching any subfilter.
    Any {
        /// Corresponds to `PHPickerFilter.anyFilterMatchingSubfilters(_:)`.
        subfilters: Vec<Self>,
    },
    /// Filter matching all subfilters.
    All {
        /// Corresponds to `PHPickerFilter.allFilterMatchingSubfilters(_:)`.
        subfilters: Vec<Self>,
    },
    /// Filter negating a subfilter.
    Not {
        /// Corresponds to `PHPickerFilter.notFilterOfSubfilter(_:)`.
        subfilter: Box<Self>,
    },
}

impl PHPickerFilter {
    /// Returns the `PHPickerFilter.imagesFilter` preset.
    pub const fn images() -> Self {
        Self::Images
    }

    /// Returns the `PHPickerFilter.videosFilter` preset.
    pub const fn videos() -> Self {
        Self::Videos
    }

    /// Returns the `PHPickerFilter.livePhotosFilter` preset.
    pub const fn live_photos() -> Self {
        Self::LivePhotos
    }

    /// Returns the `PHPickerFilter.depthEffectPhotosFilter` preset.
    pub const fn depth_effect_photos() -> Self {
        Self::DepthEffectPhotos
    }

    /// Returns the `PHPickerFilter.burstsFilter` preset.
    pub const fn bursts() -> Self {
        Self::Bursts
    }

    /// Returns the `PHPickerFilter.panoramasFilter` preset.
    pub const fn panoramas() -> Self {
        Self::Panoramas
    }

    /// Returns the `PHPickerFilter.screenshotsFilter` preset.
    pub const fn screenshots() -> Self {
        Self::Screenshots
    }

    /// Returns the `PHPickerFilter.screenRecordingsFilter` preset.
    pub const fn screen_recordings() -> Self {
        Self::ScreenRecordings
    }

    /// Returns the `PHPickerFilter.cinematicVideosFilter` preset.
    pub const fn cinematic_videos() -> Self {
        Self::CinematicVideos
    }

    /// Returns the `PHPickerFilter.slomoVideosFilter` preset.
    pub const fn slomo_videos() -> Self {
        Self::SlomoVideos
    }

    /// Returns the `PHPickerFilter.timelapseVideosFilter` preset.
    pub const fn timelapse_videos() -> Self {
        Self::TimelapseVideos
    }

    /// Returns the `PHPickerFilter.spatialMediaFilter` preset.
    pub const fn spatial_media() -> Self {
        Self::SpatialMedia
    }

    /// Returns a playback-style filter.
    pub const fn playback_style(playback_style: PHAssetPlaybackStyle) -> Self {
        Self::PlaybackStyle { playback_style }
    }

    /// Returns a filter matching any of the provided subfilters.
    pub fn any_matching(subfilters: Vec<Self>) -> Self {
        Self::Any { subfilters }
    }

    /// Returns a filter matching all of the provided subfilters.
    pub fn all_matching(subfilters: Vec<Self>) -> Self {
        Self::All { subfilters }
    }

    /// Returns a filter negating the provided subfilter.
    pub fn not_matching(subfilter: Self) -> Self {
        Self::Not {
            subfilter: Box::new(subfilter),
        }
    }

    /// Returns whether `PHPickerFilter` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_picker_filter_is_available() == ffi::status::OK }
    }

    /// Returns the PhotosUI debug description of this filter.
    pub fn description(&self) -> Result<String, PhotoKitError> {
        let filter_json = json_cstring(self, "PHPickerFilter")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_picker_filter_description_json(filter_json.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "picker filter description failed") })
        } else {
            unsafe { take_string(payload) }.ok_or_else(|| {
                PhotoKitError::OperationFailed("missing PHPickerFilter description".to_owned())
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerConfiguration`.
pub struct PHPickerConfiguration {
    /// Corresponds to `PHPickerConfiguration.preferredAssetRepresentationMode`.
    pub preferred_asset_representation_mode: PHPickerConfigurationAssetRepresentationMode,
    /// Corresponds to `PHPickerConfiguration.selection`.
    pub selection: PHPickerConfigurationSelection,
    /// Corresponds to `PHPickerConfiguration.selectionLimit`.
    pub selection_limit: i64,
    /// Corresponds to `PHPickerConfiguration.filter`.
    pub filter: Option<PHPickerFilter>,
    #[serde(default)]
    /// Corresponds to `PHPickerConfiguration.preselectedAssetIdentifiers`.
    pub preselected_asset_identifiers: Vec<String>,
    /// Corresponds to `PHPickerConfiguration.mode`.
    pub mode: Option<PHPickerMode>,
    /// Corresponds to `PHPickerConfiguration.edgesWithoutContentMargins`.
    pub edges_without_content_margins: Option<PHDirectionalRectEdge>,
    /// Corresponds to `PHPickerConfiguration.disabledCapabilities`.
    pub disabled_capabilities: Option<PHPickerCapabilities>,
}

impl Default for PHPickerConfiguration {
    fn default() -> Self {
        Self {
            preferred_asset_representation_mode:
                PHPickerConfigurationAssetRepresentationMode::Automatic,
            selection: PHPickerConfigurationSelection::Default,
            selection_limit: 1,
            filter: None,
            preselected_asset_identifiers: Vec::new(),
            mode: None,
            edges_without_content_margins: None,
            disabled_capabilities: None,
        }
    }
}

impl PHPickerConfiguration {
    /// Creates a default configuration for `PHPickerViewController`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether `PHPickerConfiguration` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_picker_configuration_is_available() == ffi::status::OK }
    }

    /// Updates `preferredAssetRepresentationMode`.
    pub fn set_preferred_asset_representation_mode(
        mut self,
        preferred_asset_representation_mode: PHPickerConfigurationAssetRepresentationMode,
    ) -> Self {
        self.preferred_asset_representation_mode = preferred_asset_representation_mode;
        self
    }

    /// Updates `selection`.
    pub fn set_selection(mut self, selection: PHPickerConfigurationSelection) -> Self {
        self.selection = selection;
        self
    }

    /// Updates `selectionLimit`.
    pub fn set_selection_limit(mut self, selection_limit: i64) -> Self {
        self.selection_limit = selection_limit;
        self
    }

    /// Updates `filter`.
    pub fn set_filter(mut self, filter: PHPickerFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Clears `filter`.
    pub fn clear_filter(mut self) -> Self {
        self.filter = None;
        self
    }

    /// Updates `preselectedAssetIdentifiers`.
    pub fn set_preselected_asset_identifiers(
        mut self,
        preselected_asset_identifiers: Vec<String>,
    ) -> Self {
        self.preselected_asset_identifiers = preselected_asset_identifiers;
        self
    }

    /// Updates `mode`.
    pub fn set_mode(mut self, mode: PHPickerMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Clears `mode`.
    pub fn clear_mode(mut self) -> Self {
        self.mode = None;
        self
    }

    /// Updates `edgesWithoutContentMargins`.
    pub fn set_edges_without_content_margins(
        mut self,
        edges_without_content_margins: PHDirectionalRectEdge,
    ) -> Self {
        self.edges_without_content_margins = Some(edges_without_content_margins);
        self
    }

    /// Clears `edgesWithoutContentMargins`.
    pub fn clear_edges_without_content_margins(mut self) -> Self {
        self.edges_without_content_margins = None;
        self
    }

    /// Updates `disabledCapabilities`.
    pub fn set_disabled_capabilities(
        mut self,
        disabled_capabilities: PHPickerCapabilities,
    ) -> Self {
        self.disabled_capabilities = Some(disabled_capabilities);
        self
    }

    /// Clears `disabledCapabilities`.
    pub fn clear_disabled_capabilities(mut self) -> Self {
        self.disabled_capabilities = None;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerUpdateConfiguration`.
pub struct PHPickerUpdateConfiguration {
    /// Corresponds to `PHPickerUpdateConfiguration.selectionLimit`.
    pub selection_limit: Option<i64>,
    /// Corresponds to `PHPickerUpdateConfiguration.edgesWithoutContentMargins`.
    pub edges_without_content_margins: Option<PHDirectionalRectEdge>,
}

impl PHPickerUpdateConfiguration {
    /// Creates an empty update configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates `selectionLimit`.
    pub fn set_selection_limit(mut self, selection_limit: i64) -> Self {
        self.selection_limit = Some(selection_limit);
        self
    }

    /// Updates `edgesWithoutContentMargins`.
    pub fn set_edges_without_content_margins(
        mut self,
        edges_without_content_margins: PHDirectionalRectEdge,
    ) -> Self {
        self.edges_without_content_margins = Some(edges_without_content_margins);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Summary of the `NSItemProvider` embedded in `PHPickerResult`.
pub struct PHItemProviderInfo {
    /// Corresponds to `NSItemProvider.registeredTypeIdentifiers`.
    pub registered_type_identifiers: Vec<String>,
    /// Corresponds to `NSItemProvider.suggestedName`.
    pub suggested_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPickerResult`.
pub struct PHPickerResult {
    /// Corresponds to `PHPickerResult.assetIdentifier`.
    pub asset_identifier: Option<String>,
    /// Corresponds to `PHPickerResult.itemProvider`.
    pub item_provider: PHItemProviderInfo,
}

impl PHPickerResult {
    /// Returns whether `PHPickerResult` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_picker_result_is_available() == ffi::status::OK }
    }
}

type PickerDelegateCallback = dyn Fn(Vec<PHPickerResult>) + Send;

#[derive(Debug)]
/// Wraps `PHPickerViewController`.
pub struct PHPickerViewController {
    raw: NonNull<c_void>,
    configuration: PHPickerConfiguration,
}

impl PHPickerViewController {
    /// Returns whether `PHPickerViewController` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_picker_view_controller_is_available() == ffi::status::OK }
    }

    /// Creates a new picker backed by the system photo library.
    pub fn new(configuration: &PHPickerConfiguration) -> Result<Self, PhotoKitError> {
        Self::new_impl(configuration, None)
    }

    /// Creates a new picker backed by the supplied `PHPhotoLibrary`.
    pub fn new_with_photo_library(
        configuration: &PHPickerConfiguration,
        photo_library: &PHPhotoLibrary,
    ) -> Result<Self, PhotoKitError> {
        Self::new_impl(configuration, Some(photo_library))
    }

    fn new_impl(
        configuration: &PHPickerConfiguration,
        photo_library: Option<&PHPhotoLibrary>,
    ) -> Result<Self, PhotoKitError> {
        let configuration_json = json_cstring(configuration, "PHPickerConfiguration")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_picker_view_controller_new(
                configuration_json.as_ptr(),
                photo_library.map_or(ptr::null_mut(), |value| value.raw.as_ptr()),
                &mut error,
            )
        };
        let raw = NonNull::new(raw).ok_or_else(|| unsafe {
            PhotoKitError::from_error_ptr(error, "create PHPickerViewController failed")
        })?;
        Ok(Self {
            raw,
            configuration: configuration.clone(),
        })
    }

    /// Returns the configuration used when the picker was created.
    pub fn configuration(&self) -> &PHPickerConfiguration {
        &self.configuration
    }

    /// Updates the picker using `PHPickerUpdateConfiguration`.
    pub fn update_picker_using_configuration(
        &self,
        configuration: &PHPickerUpdateConfiguration,
    ) -> Result<(), PhotoKitError> {
        let configuration_json = json_cstring(configuration, "PHPickerUpdateConfiguration")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_picker_view_controller_update_picker_json(
                self.raw.as_ptr(),
                configuration_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "update PHPickerViewController failed")
            })
        }
    }

    /// Deselects the supplied asset identifiers in the picker.
    pub fn deselect_assets_with_identifiers(
        &self,
        identifiers: &[String],
    ) -> Result<(), PhotoKitError> {
        let identifiers_json = json_cstring(identifiers, "picker asset identifiers")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_picker_view_controller_deselect_assets_json(
                self.raw.as_ptr(),
                identifiers_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "deselect picker assets failed") })
        }
    }

    /// Moves a selected asset after another selected asset identifier.
    pub fn move_asset_with_identifier(
        &self,
        identifier: &str,
        after_identifier: Option<&str>,
    ) -> Result<(), PhotoKitError> {
        let identifier = cstring_from_str(identifier, "picker asset identifier")?;
        let after_identifier = after_identifier
            .map(|value| cstring_from_str(value, "picker after asset identifier"))
            .transpose()?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_picker_view_controller_move_asset(
                self.raw.as_ptr(),
                identifier.as_ptr(),
                after_identifier
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "move picker asset failed") })
        }
    }

    /// Scrolls the picker back to its initial position.
    pub fn scroll_to_initial_position(&self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status =
            unsafe { ffi::ph_picker_view_controller_scroll_to_initial_position(self.raw.as_ptr(), &mut error) };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "scroll picker to initial position failed")
            })
        }
    }

    /// Requests the picker to zoom in if possible.
    pub fn zoom_in(&self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe { ffi::ph_picker_view_controller_zoom_in(self.raw.as_ptr(), &mut error) };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "picker zoom-in failed") })
        }
    }

    /// Requests the picker to zoom out if possible.
    pub fn zoom_out(&self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe { ffi::ph_picker_view_controller_zoom_out(self.raw.as_ptr(), &mut error) };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "picker zoom-out failed") })
        }
    }

    /// Registers a delegate callback for picker completion events.
    pub fn register_delegate<F>(
        &self,
        callback: F,
    ) -> Result<PHPickerViewControllerDelegate, PhotoKitError>
    where
        F: Fn(Vec<PHPickerResult>) + Send + 'static,
    {
        let user_info = unsafe {
            NonNull::new_unchecked(
                Box::into_raw(Box::new(Box::new(callback) as Box<PickerDelegateCallback>))
                    .cast::<c_void>(),
            )
        };
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_picker_view_controller_register_delegate(
                self.raw.as_ptr(),
                picker_view_controller_delegate_trampoline,
                user_info.as_ptr(),
                &mut error,
            )
        };
        if let Some(raw) = NonNull::new(raw) {
            Ok(PHPickerViewControllerDelegate { raw, user_info })
        } else {
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<Box<PickerDelegateCallback>>(),
                ));
            }
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "register PHPickerViewControllerDelegate failed")
            })
        }
    }
}

impl Drop for PHPickerViewController {
    fn drop(&mut self) {
        unsafe { ffi::ph_picker_view_controller_release(self.raw.as_ptr()) };
    }
}

/// RAII registration token for `PHPickerViewControllerDelegate`.
pub struct PHPickerViewControllerDelegate {
    raw: NonNull<c_void>,
    user_info: NonNull<c_void>,
}

impl PHPickerViewControllerDelegate {
    /// Returns whether `PHPickerViewControllerDelegate` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_picker_view_controller_delegate_is_available() == ffi::status::OK }
    }
}

impl core::fmt::Debug for PHPickerViewControllerDelegate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHPickerViewControllerDelegate")
            .finish_non_exhaustive()
    }
}

impl Drop for PHPickerViewControllerDelegate {
    fn drop(&mut self) {
        unsafe { ffi::ph_picker_view_controller_unregister_delegate(self.raw.as_ptr()) };
        unsafe {
            drop(Box::from_raw(
                self.user_info.as_ptr().cast::<Box<PickerDelegateCallback>>(),
            ));
        }
    }
}

unsafe extern "C" fn picker_view_controller_delegate_trampoline(
    payload_json: *mut c_char,
    user_info: *mut c_void,
) {
    if user_info.is_null() {
        return;
    }

    let callback = &mut **user_info.cast::<Box<PickerDelegateCallback>>();
    let results = if payload_json.is_null() {
        Vec::new()
    } else if let Some(json) = take_string(payload_json) {
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        Vec::new()
    };
    catch_user_panic("picker_view_controller_delegate_trampoline", || {
        callback(results);
    });
}
