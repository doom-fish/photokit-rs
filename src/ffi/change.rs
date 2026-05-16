use core::ffi::c_void;

extern "C" {
    pub fn ph_change_release(change: *mut c_void);
}
