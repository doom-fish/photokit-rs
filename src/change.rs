use core::ffi::c_void;
use std::ptr::{self, NonNull};

use crate::asset::PHAsset;
use crate::asset_collection::PHAssetCollection;
use crate::collection_list::PHCollectionList;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::object_change_details::PHObjectChangeDetails;
use crate::private::{cstring_from_str, parse_json_ptr};

pub struct PHChange {
    pub(crate) raw: NonNull<c_void>,
}

impl PHChange {
    pub(crate) unsafe fn from_raw(raw: *mut c_void) -> Self {
        Self {
            raw: NonNull::new(raw).expect("PHChange handle should not be null"),
        }
    }

    pub fn asset_change_details(
        &self,
        asset: &PHAsset,
    ) -> Result<Option<PHObjectChangeDetails<PHAsset>>, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_change_asset_change_details_json(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset change detail lookup failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHObjectChangeDetails<PHAsset>") }
        }
    }

    pub fn asset_collection_change_details(
        &self,
        collection: &PHAssetCollection,
    ) -> Result<Option<PHObjectChangeDetails<PHAssetCollection>>, PhotoKitError> {
        let collection_identifier =
            cstring_from_str(&collection.local_identifier, "collection local identifier")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_change_asset_collection_change_details_json(
                self.raw.as_ptr(),
                collection_identifier.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "asset collection change detail lookup failed",
                )
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHObjectChangeDetails<PHAssetCollection>") }
        }
    }

    pub fn collection_list_change_details(
        &self,
        collection_list: &PHCollectionList,
    ) -> Result<Option<PHObjectChangeDetails<PHCollectionList>>, PhotoKitError> {
        let collection_identifier = cstring_from_str(
            &collection_list.local_identifier,
            "collection list local identifier",
        )?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_change_collection_list_change_details_json(
                self.raw.as_ptr(),
                collection_identifier.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "collection list change detail lookup failed",
                )
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHObjectChangeDetails<PHCollectionList>") }
        }
    }
}

impl core::fmt::Debug for PHChange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHChange").finish_non_exhaustive()
    }
}

impl Drop for PHChange {
    fn drop(&mut self) {
        unsafe { ffi::ph_change_release(self.raw.as_ptr()) };
    }
}
