use core::fmt;
use std::ffi::CStr;

use serde::{Deserialize, Serialize};

use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PHAuthorizationStatus {
    NotDetermined,
    Restricted,
    Denied,
    Authorized,
    Limited,
    Unknown(i32),
}

impl PHAuthorizationStatus {
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::NotDetermined,
            1 => Self::Restricted,
            2 => Self::Denied,
            3 => Self::Authorized,
            4 => Self::Limited,
            other => Self::Unknown(other),
        }
    }

    pub const fn is_authorized(self) -> bool {
        matches!(self, Self::Authorized | Self::Limited)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PHPhotosError(pub i64);

impl PHPhotosError {
    pub const INTERNAL_ERROR: Self = Self(-1);
    pub const USER_CANCELLED: Self = Self(3072);
    pub const LIBRARY_VOLUME_OFFLINE: Self = Self(3114);
    pub const RELINQUISHING_LIBRARY_BUNDLE_TO_WRITER: Self = Self(3142);
    pub const SWITCHING_SYSTEM_PHOTO_LIBRARY: Self = Self(3143);
    pub const NETWORK_ACCESS_REQUIRED: Self = Self(3164);
    pub const NETWORK_ERROR: Self = Self(3169);
    pub const IDENTIFIER_NOT_FOUND: Self = Self(3201);
    pub const MULTIPLE_IDENTIFIERS_FOUND: Self = Self(3202);
    pub const CHANGE_NOT_SUPPORTED: Self = Self(3300);
    pub const OPERATION_INTERRUPTED: Self = Self(3301);
    pub const INVALID_RESOURCE: Self = Self(3302);
    pub const MISSING_RESOURCE: Self = Self(3303);
    pub const NOT_ENOUGH_SPACE: Self = Self(3305);
    pub const REQUEST_NOT_SUPPORTED_FOR_ASSET: Self = Self(3306);
    pub const LIMIT_EXCEEDED: Self = Self(3307);
    pub const ACCESS_RESTRICTED: Self = Self(3310);
    pub const ACCESS_USER_DENIED: Self = Self(3311);
    pub const LIBRARY_IN_FILE_PROVIDER_SYNC_ROOT: Self = Self(5423);
    pub const PERSISTENT_CHANGE_TOKEN_EXPIRED: Self = Self(3105);
    pub const PERSISTENT_CHANGE_DETAILS_UNAVAILABLE: Self = Self(3210);

    pub const fn raw_value(self) -> i64 {
        self.0
    }
}

#[allow(non_upper_case_globals)]
pub const PHPhotosErrorDomain: &str = "PHPhotosErrorDomain";
#[allow(non_upper_case_globals)]
pub const PHLocalIdentifiersErrorKey: &str = "PHLocalIdentifiersErrorKey";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NSErrorInfo {
    pub domain: String,
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub local_identifiers: Vec<String>,
}

impl NSErrorInfo {
    pub fn photos_error(&self) -> Option<PHPhotosError> {
        (self.domain == PHPhotosErrorDomain).then_some(PHPhotosError(self.code))
    }
}

impl fmt::Display for NSErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) [{}]", self.message, self.code, self.domain)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhotoKitError {
    InvalidArgument(String),
    Framework(NSErrorInfo),
    OperationFailed(String),
}

impl fmt::Display for PhotoKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::Framework(error) => write!(f, "Photos.framework error: {error}"),
            Self::OperationFailed(message) => write!(f, "photokit operation failed: {message}"),
        }
    }
}

impl std::error::Error for PhotoKitError {}

impl PhotoKitError {
    pub fn framework_error(&self) -> Option<&NSErrorInfo> {
        match self {
            Self::Framework(error) => Some(error),
            _ => None,
        }
    }

    pub fn photos_error(&self) -> Option<PHPhotosError> {
        self.framework_error().and_then(NSErrorInfo::photos_error)
    }

    pub(crate) unsafe fn from_error_ptr(error_ptr: *mut core::ffi::c_char, fallback: &str) -> Self {
        if error_ptr.is_null() {
            return Self::OperationFailed(fallback.to_owned());
        }

        let message = CStr::from_ptr(error_ptr).to_string_lossy().into_owned();
        ffi::ph_string_free(error_ptr);

        if let Ok(payload) = serde_json::from_str::<NSErrorInfo>(&message) {
            Self::Framework(payload)
        } else {
            Self::OperationFailed(message)
        }
    }
}
