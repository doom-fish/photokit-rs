use core::fmt;
use std::ffi::CStr;

use serde::{Deserialize, Serialize};

use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
/// Wraps `PHAuthorizationStatus`.
pub enum PHAuthorizationStatus {
    /// Case of `PHAuthorizationStatus`.
    NotDetermined,
    /// Case of `PHAuthorizationStatus`.
    Restricted,
    /// Case of `PHAuthorizationStatus`.
    Denied,
    /// Case of `PHAuthorizationStatus`.
    Authorized,
    /// Case of `PHAuthorizationStatus`.
    Limited,
    /// Case of `PHAuthorizationStatus`.
    Unknown(
        /// Associated value for `PHAuthorizationStatus::Unknown`.
        i32,
    ),
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

    /// Returns whether the Photos framework treats this `PHAuthorizationStatus` as authorized.
    pub const fn is_authorized(self) -> bool {
        matches!(self, Self::Authorized | Self::Limited)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Wraps `PHPhotosError`.
pub struct PHPhotosError(
    /// Raw value for `PHPhotosError`.
    pub i64,
);

impl PHPhotosError {
    /// Constant on `PHPhotosError`.
    pub const INTERNAL_ERROR: Self = Self(-1);
    /// Constant on `PHPhotosError`.
    pub const USER_CANCELLED: Self = Self(3072);
    /// Constant on `PHPhotosError`.
    pub const LIBRARY_VOLUME_OFFLINE: Self = Self(3114);
    /// Constant on `PHPhotosError`.
    pub const RELINQUISHING_LIBRARY_BUNDLE_TO_WRITER: Self = Self(3142);
    /// Constant on `PHPhotosError`.
    pub const SWITCHING_SYSTEM_PHOTO_LIBRARY: Self = Self(3143);
    /// Constant on `PHPhotosError`.
    pub const NETWORK_ACCESS_REQUIRED: Self = Self(3164);
    /// Constant on `PHPhotosError`.
    pub const NETWORK_ERROR: Self = Self(3169);
    /// Constant on `PHPhotosError`.
    pub const IDENTIFIER_NOT_FOUND: Self = Self(3201);
    /// Constant on `PHPhotosError`.
    pub const MULTIPLE_IDENTIFIERS_FOUND: Self = Self(3202);
    /// Constant on `PHPhotosError`.
    pub const CHANGE_NOT_SUPPORTED: Self = Self(3300);
    /// Constant on `PHPhotosError`.
    pub const OPERATION_INTERRUPTED: Self = Self(3301);
    /// Constant on `PHPhotosError`.
    pub const INVALID_RESOURCE: Self = Self(3302);
    /// Constant on `PHPhotosError`.
    pub const MISSING_RESOURCE: Self = Self(3303);
    /// Constant on `PHPhotosError`.
    pub const NOT_ENOUGH_SPACE: Self = Self(3305);
    /// Constant on `PHPhotosError`.
    pub const REQUEST_NOT_SUPPORTED_FOR_ASSET: Self = Self(3306);
    /// Constant on `PHPhotosError`.
    pub const LIMIT_EXCEEDED: Self = Self(3307);
    /// Constant on `PHPhotosError`.
    pub const ACCESS_RESTRICTED: Self = Self(3310);
    /// Constant on `PHPhotosError`.
    pub const ACCESS_USER_DENIED: Self = Self(3311);
    /// Constant on `PHPhotosError`.
    pub const LIBRARY_IN_FILE_PROVIDER_SYNC_ROOT: Self = Self(5423);
    /// Constant on `PHPhotosError`.
    pub const PERSISTENT_CHANGE_TOKEN_EXPIRED: Self = Self(3105);
    /// Constant on `PHPhotosError`.
    pub const PERSISTENT_CHANGE_DETAILS_UNAVAILABLE: Self = Self(3210);

    /// Returns the raw Photos framework value for `PHPhotosError`.
    pub const fn raw_value(self) -> i64 {
        self.0
    }
}

#[allow(non_upper_case_globals)]
/// Matches `PHPhotosErrorDomain`.
pub const PHPhotosErrorDomain: &str = "PHPhotosErrorDomain";
#[allow(non_upper_case_globals)]
/// Matches `PHLocalIdentifiersErrorKey`.
pub const PHLocalIdentifiersErrorKey: &str = "PHLocalIdentifiersErrorKey";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized `NSError` information returned by Photos framework calls.
pub struct NSErrorInfo {
    /// Corresponds to `NSErrorInfo.domain`.
    pub domain: String,
    /// Corresponds to `NSErrorInfo.code`.
    pub code: i64,
    /// Corresponds to `NSErrorInfo.message`.
    pub message: String,
    #[serde(default)]
    /// Corresponds to `NSErrorInfo.localIdentifiers`.
    pub local_identifiers: Vec<String>,
}

impl NSErrorInfo {
    /// Wraps a Photos framework operation on `NSErrorInfo`.
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
/// Error type for Photos framework wrapper operations.
pub enum PhotoKitError {
    /// Indicates that a Photos framework wrapper argument was invalid.
    InvalidArgument(
        /// Associated value for `PhotoKitError::InvalidArgument`.
        String,
    ),
    /// Wraps an `NSError` returned by a Photos framework call.
    Framework(
        /// Associated value for `PhotoKitError::Framework`.
        NSErrorInfo,
    ),
    /// Indicates that a Photos framework wrapper operation failed before an `NSError` payload was available.
    OperationFailed(
        /// Associated value for `PhotoKitError::OperationFailed`.
        String,
    ),
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
    /// Wraps a Photos framework operation on `PhotoKitError`.
    pub fn framework_error(&self) -> Option<&NSErrorInfo> {
        match self {
            Self::Framework(error) => Some(error),
            _ => None,
        }
    }

    /// Wraps a Photos framework operation on `PhotoKitError`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_status_maps_known_raw_values() {
        let cases = [
            (0, PHAuthorizationStatus::NotDetermined),
            (1, PHAuthorizationStatus::Restricted),
            (2, PHAuthorizationStatus::Denied),
            (3, PHAuthorizationStatus::Authorized),
            (4, PHAuthorizationStatus::Limited),
        ];

        for (raw, expected) in cases {
            assert_eq!(PHAuthorizationStatus::from_raw(raw), expected);
        }
    }

    #[test]
    fn authorization_status_preserves_unknown_values() {
        assert_eq!(
            PHAuthorizationStatus::from_raw(42),
            PHAuthorizationStatus::Unknown(42)
        );
    }

    #[test]
    fn authorization_status_reports_authorized_variants() {
        assert!(PHAuthorizationStatus::Authorized.is_authorized());
        assert!(PHAuthorizationStatus::Limited.is_authorized());
        assert!(!PHAuthorizationStatus::Denied.is_authorized());
    }

    #[test]
    fn photos_error_constants_expose_expected_raw_values() {
        assert_eq!(PHPhotosError::USER_CANCELLED.raw_value(), 3_072);
        assert_eq!(PHPhotosError::NETWORK_ERROR.raw_value(), 3_169);
        assert_eq!(PHPhotosError::ACCESS_USER_DENIED.raw_value(), 3_311);
    }

    #[test]
    fn ns_error_info_only_maps_photos_domain() {
        let framework_error = NSErrorInfo {
            domain: PHPhotosErrorDomain.into(),
            code: PHPhotosError::NETWORK_ERROR.raw_value(),
            message: "network unavailable".into(),
            local_identifiers: vec!["A".into(), "B".into()],
        };
        let other_error = NSErrorInfo {
            domain: "OtherDomain".into(),
            code: 7,
            message: "other".into(),
            local_identifiers: Vec::new(),
        };

        assert_eq!(
            framework_error.photos_error(),
            Some(PHPhotosError::NETWORK_ERROR)
        );
        assert!(other_error.photos_error().is_none());
        assert_eq!(
            framework_error.to_string(),
            "network unavailable (3169) [PHPhotosErrorDomain]"
        );
    }

    #[test]
    fn photokit_error_accessors_surface_wrapped_framework_error() {
        let framework_error = NSErrorInfo {
            domain: PHPhotosErrorDomain.into(),
            code: PHPhotosError::LIMIT_EXCEEDED.raw_value(),
            message: "too many items".into(),
            local_identifiers: vec!["asset-1".into()],
        };
        let error = PhotoKitError::Framework(framework_error.clone());

        assert_eq!(error.framework_error(), Some(&framework_error));
        assert_eq!(error.photos_error(), Some(PHPhotosError::LIMIT_EXCEEDED));
        assert_eq!(
            error.to_string(),
            "Photos.framework error: too many items (3307) [PHPhotosErrorDomain]"
        );
    }
}
