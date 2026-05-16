#![allow(clippy::missing_errors_doc)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::PhotoKitError;
use crate::ffi;

pub fn cstring_from_str(value: &str, context: &str) -> Result<CString, PhotoKitError> {
    CString::new(value).map_err(|error| {
        PhotoKitError::InvalidArgument(format!("{context} contains NUL byte: {error}"))
    })
}

pub fn json_cstring<T: Serialize + ?Sized>(
    value: &T,
    context: &str,
) -> Result<CString, PhotoKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        PhotoKitError::InvalidArgument(format!("failed to encode {context} as JSON: {error}"))
    })?;
    cstring_from_str(&json, context)
}

pub unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    let string = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::ph_string_free(ptr);
    Some(string)
}

pub unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, PhotoKitError> {
    let json = take_string(ptr).ok_or_else(|| {
        PhotoKitError::OperationFailed(format!("missing JSON payload for {context}"))
    })?;

    serde_json::from_str(&json).map_err(|error| {
        PhotoKitError::OperationFailed(format!(
            "failed to parse {context} JSON: {error}; payload={json}"
        ))
    })
}
