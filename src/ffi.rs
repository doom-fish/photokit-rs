#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

pub use doom_fish_utils::ffi_callbacks::JsonCallback;

extern "C" {
    pub fn ph_string_free(string: *mut c_char);
}

pub type ChangeObserverCallback = unsafe extern "C" fn(change: *mut c_void, user_info: *mut c_void);
pub type LivePhotoFrameProcessorCallback =
    unsafe extern "C" fn(frame_json: *const c_char, user_info: *mut c_void) -> i32;
pub type BoolJsonCallback = unsafe extern "C" fn(json: *mut c_char, user_info: *mut c_void) -> i32;

pub mod asset;
pub mod asset_change_request;
pub mod asset_collection;
pub mod asset_collection_change_request;
pub mod asset_creation_request;
pub mod asset_resource_manager;
#[cfg(feature = "async")]
pub mod async_ffi;
pub mod change;
pub mod cloud_identifier;
pub mod collection;
pub mod collection_list;
pub mod collection_list_change_request;
pub mod content_editing_controller;
pub mod content_editing_input;
pub mod content_editing_output;
pub mod fetch_options;
pub mod fetch_result;
pub mod image_manager;
pub mod live_photo;
pub mod live_photo_editing_context;
pub mod live_photo_view;
pub mod object_change_details;
pub mod photo_library;
pub mod picker;
pub mod project;
pub mod project_extension;
pub mod project_info;

pub use asset::*;
pub use asset_change_request::*;
pub use asset_collection::*;
pub use asset_collection_change_request::*;
pub use asset_creation_request::*;
pub use asset_resource_manager::*;
#[cfg(feature = "async")]
pub use async_ffi::*;
pub use change::*;
pub use cloud_identifier::*;
pub use collection::*;
pub use collection_list::*;
pub use collection_list_change_request::*;
pub use content_editing_controller::*;
pub use content_editing_input::*;
pub use content_editing_output::*;
pub use image_manager::*;
pub use live_photo::*;
pub use live_photo_editing_context::*;
pub use live_photo_view::*;
pub use object_change_details::*;
pub use photo_library::*;
pub use picker::*;
pub use project::*;
pub use project_extension::*;
pub use project_info::*;

pub mod status {
    pub const OK: i32 = 0;
}
