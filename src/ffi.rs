#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_string_free(string: *mut c_char);
}

pub type ChangeObserverCallback = unsafe extern "C" fn(change: *mut c_void, user_info: *mut c_void);

pub mod asset;
pub mod asset_collection;
pub mod asset_creation_request;
pub mod change;
pub mod cloud_identifier;
pub mod collection_list;
pub mod content_editing_input;
pub mod content_editing_output;
pub mod fetch_options;
pub mod fetch_result;
pub mod image_manager;
pub mod live_photo;
pub mod object_change_details;
pub mod photo_library;

pub use asset::*;
pub use asset_collection::*;
pub use asset_creation_request::*;
pub use change::*;
pub use cloud_identifier::*;
pub use collection_list::*;
pub use content_editing_input::*;
pub use content_editing_output::*;
pub use image_manager::*;
pub use live_photo::*;
pub use object_change_details::*;
pub use photo_library::*;

pub mod status {
    pub const OK: i32 = 0;
}
