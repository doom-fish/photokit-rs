use base64::Engine;

use crate::content_editing_input::{PHAdjustmentData, PHContentEditingInput};
use crate::content_editing_output::PHContentEditingOutput;
use crate::error::PhotoKitError;
use crate::ffi;

type CanHandleAdjustmentDataCallback = dyn Fn(&PHAdjustmentData) -> bool + Send + Sync;
type StartContentEditingCallback =
    dyn Fn(&PHContentEditingInput, &PHContentEditingPlaceholderImage) + Send + Sync;
type FinishContentEditingCallback =
    dyn Fn() -> Result<Option<PHContentEditingOutput>, PhotoKitError> + Send + Sync;
type CancelContentEditingCallback = dyn Fn() + Send + Sync;
type ShouldShowCancelConfirmationCallback = dyn Fn() -> bool + Send + Sync;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Serialized placeholder image delivered with `PHContentEditingController` startup.
pub struct PHContentEditingPlaceholderImage {
    /// Serialized TIFF payload for the placeholder image.
    pub tiff_data_base64: String,
    /// Pixel width of the placeholder image.
    pub width: f64,
    /// Pixel height of the placeholder image.
    pub height: f64,
}

impl PHContentEditingPlaceholderImage {
    /// Decodes the TIFF payload carried by the placeholder image.
    pub fn tiff_data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.tiff_data_base64.as_bytes())
            .unwrap_or_default()
    }
}

/// Rust-side wrapper matching the `PHContentEditingController` protocol contract.
pub struct PHContentEditingController {
    can_handle_adjustment_data: Box<CanHandleAdjustmentDataCallback>,
    start_content_editing: Box<StartContentEditingCallback>,
    finish_content_editing: Box<FinishContentEditingCallback>,
    cancel_content_editing: Box<CancelContentEditingCallback>,
    should_show_cancel_confirmation: Box<ShouldShowCancelConfirmationCallback>,
}

impl PHContentEditingController {
    /// Creates a content-editing controller helper.
    pub fn new<CanHandle, Start, Finish, Cancel, Confirm>(
        can_handle_adjustment_data_callback: CanHandle,
        start_content_editing_callback: Start,
        finish_content_editing_callback: Finish,
        cancel_content_editing_callback: Cancel,
        should_show_cancel_confirmation_callback: Confirm,
    ) -> Self
    where
        CanHandle: Fn(&PHAdjustmentData) -> bool + Send + Sync + 'static,
        Start: Fn(&PHContentEditingInput, &PHContentEditingPlaceholderImage) + Send + Sync + 'static,
        Finish: Fn() -> Result<Option<PHContentEditingOutput>, PhotoKitError>
            + Send
            + Sync
            + 'static,
        Cancel: Fn() + Send + Sync + 'static,
        Confirm: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            can_handle_adjustment_data: Box::new(can_handle_adjustment_data_callback),
            start_content_editing: Box::new(start_content_editing_callback),
            finish_content_editing: Box::new(finish_content_editing_callback),
            cancel_content_editing: Box::new(cancel_content_editing_callback),
            should_show_cancel_confirmation: Box::new(
                should_show_cancel_confirmation_callback,
            ),
        }
    }

    /// Returns whether `PHContentEditingController` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_content_editing_controller_is_available() == ffi::status::OK }
    }

    /// Wraps `canHandleAdjustmentData(_:)`.
    pub fn can_handle_adjustment_data(&self, adjustment_data: &PHAdjustmentData) -> bool {
        (self.can_handle_adjustment_data)(adjustment_data)
    }

    /// Wraps `startContentEditing(with:placeholderImage:)`.
    pub fn start_content_editing_with_input(
        &self,
        content_editing_input: &PHContentEditingInput,
        placeholder_image: &PHContentEditingPlaceholderImage,
    ) {
        (self.start_content_editing)(content_editing_input, placeholder_image);
    }

    /// Wraps `finishContentEditingWithCompletionHandler(_:)`.
    pub fn finish_content_editing(&self) -> Result<Option<PHContentEditingOutput>, PhotoKitError> {
        (self.finish_content_editing)()
    }

    /// Wraps `cancelContentEditing()`.
    pub fn cancel_content_editing(&self) {
        (self.cancel_content_editing)();
    }

    /// Wraps `shouldShowCancelConfirmation`.
    pub fn should_show_cancel_confirmation(&self) -> bool {
        (self.should_show_cancel_confirmation)()
    }
}

impl core::fmt::Debug for PHContentEditingController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHContentEditingController")
            .finish_non_exhaustive()
    }
}
