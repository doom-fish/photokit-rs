use std::ops::Deref;
use std::ptr;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::asset_collection::PHAssetCollection;
use crate::change_request::PHChangeRequest;
use crate::collection::PHCollection;
use crate::error::PhotoKitError;
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::private::{json_cstring, parse_json_ptr};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHProject {
    #[serde(flatten)]
    pub asset_collection: PHAssetCollection,
    pub project_extension_data_base64: String,
    #[serde(default)]
    pub has_project_preview: bool,
}

impl PHProject {
    pub fn fetch_top_level_user_collections(
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_project_fetch_top_level_json(options_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch projects failed") })
        } else {
            let projects: Vec<Self> = unsafe { parse_json_ptr(payload, "PHProject list") }?;
            Ok(projects.into())
        }
    }

    pub fn fetch_with_local_identifiers(
        identifiers: &[String],
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let identifiers_json = json_cstring(identifiers, "project identifiers")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_project_fetch_with_local_identifiers_json(
                identifiers_json.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch projects by identifier failed")
            })
        } else {
            let projects: Vec<Self> = unsafe { parse_json_ptr(payload, "PHProject list") }?;
            Ok(projects.into())
        }
    }

    pub fn from_local_identifier(
        local_identifier: impl Into<String>,
    ) -> Result<Option<Self>, PhotoKitError> {
        let result = Self::fetch_with_local_identifiers(
            &[local_identifier.into()],
            &PHFetchOptions::default(),
        )?;
        Ok(result.into_vec().into_iter().next())
    }

    pub fn project_extension_data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.project_extension_data_base64.as_bytes())
            .unwrap_or_default()
    }

    pub fn collection(&self) -> PHCollection {
        PHCollection {
            local_identifier: self.local_identifier.clone(),
            localized_title: self.localized_title.clone(),
            can_contain_assets: self.can_contain_assets,
            can_contain_collections: self.can_contain_collections,
            kind: "project".to_owned(),
        }
    }
}

impl Deref for PHProject {
    type Target = PHAssetCollection;

    fn deref(&self) -> &Self::Target {
        &self.asset_collection
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHProjectChangeRequest {
    pub project_local_identifier: String,
    pub title: Option<String>,
    pub project_extension_data_base64: Option<String>,
    pub project_preview_image_file_url: Option<String>,
    #[serde(default)]
    pub remove_asset_identifiers: Vec<String>,
}

impl PHProjectChangeRequest {
    pub fn change_request_for_project(project: &PHProject) -> Self {
        Self {
            project_local_identifier: project.local_identifier.clone(),
            ..Self::default()
        }
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn set_project_extension_data(mut self, data_base64: impl Into<String>) -> Self {
        self.project_extension_data_base64 = Some(data_base64.into());
        self
    }

    pub fn set_project_extension_data_bytes(mut self, data: &[u8]) -> Self {
        self.project_extension_data_base64 =
            Some(base64::engine::general_purpose::STANDARD.encode(data));
        self
    }

    pub fn set_project_preview_image_file_url(mut self, file_url: impl Into<String>) -> Self {
        self.project_preview_image_file_url = Some(file_url.into());
        self
    }

    pub fn remove_assets(mut self, assets: &[PHAsset]) -> Self {
        self.remove_asset_identifiers = assets
            .iter()
            .map(|asset| asset.local_identifier.clone())
            .collect();
        self
    }
}

impl PHChangeRequest for PHProjectChangeRequest {
    type Output = ();

    fn perform(self) -> Result<Self::Output, PhotoKitError> {
        let payload_json = json_cstring(&self, "PHProjectChangeRequest")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_project_change_request_perform_json(payload_json.as_ptr(), &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "project change request failed") })
        }
    }
}

impl From<&PHProject> for PHCollection {
    fn from(value: &PHProject) -> Self {
        value.collection()
    }
}
