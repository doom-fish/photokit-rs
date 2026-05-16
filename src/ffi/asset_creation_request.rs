use core::ffi::c_char;

extern "C" {
    pub fn ph_asset_creation_request_supports_resource_types(
        resource_types_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_asset_creation_request_perform(
        resources_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
