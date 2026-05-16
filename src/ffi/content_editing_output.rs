use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_content_editing_output_new_for_input(
        input: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_content_editing_output_json(
        output: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_content_editing_output_release(output: *mut c_void);
    pub fn ph_content_editing_output_set_adjustment_data_json(
        output: *mut c_void,
        adjustment_data_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_content_editing_output_rendered_content_url_for_type(
        output: *mut c_void,
        type_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
