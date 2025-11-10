use super::component::{Component, Update};
use crate::launcher::http;
use crate::launcher::module;
use crate::launcher::sha1;
use crate::launcher::updater::github;
use core::ffi::{c_char, CStr};
use semver::Version;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::sync::Arc;

pub struct Mss32Component {
    release_information: Arc<github::ReleaseInformation>,
}

impl Mss32Component {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            release_information: Arc::new(github::fetch_release_information(
                "callofduty4x/CoD4x-mss",
            )?),
        })
    }

    pub fn get_module_version() -> Option<Version> {
        let module_path = module::get_path();
        let install_dir = module_path.parent()?;

        let mss_path = install_dir.join("mss32.dll");
        let version = unsafe {
            let Ok(mss32) = libloading::Library::new(mss_path) else {
                return None;
            };
            type TGetMss32Version = unsafe extern "C" fn() -> *const c_char;
            let Ok(get_mss32_version) = mss32.get::<TGetMss32Version>(b"get_mss32_version\0")
            else {
                return None;
            };

            let Ok(version_str) = CStr::from_ptr(get_mss32_version()).to_str() else {
                return None;
            };
            let mut version_str = version_str.to_string();
            if version_str.matches('.').count() < 2 {
                version_str.push_str(".0");
            }
            version_str
        };
        Version::parse(version.as_str()).ok()
    }

    fn get_module_update(
        &self,
        pattern: &str,
        display_name: &str,
    ) -> anyhow::Result<Option<Update>> {
        let upstream = Version::parse(self.release_information.as_ref().tag_name.as_str())?;
        let current = Self::get_module_version();

        let needs_update = current.as_ref().is_none_or(|c| upstream > *c);
        if needs_update {
            Ok(Some(Update {
                display_name: display_name.to_string(),
                artifact_name: pattern.to_string(),
                current,
                upstream,
                requires_elevate: true,
                requires_restart: true,
            }))
        } else {
            Ok(None)
        }
    }

    fn update_artifact(
        &self,
        artifact: &Update,
        hashes: &HashMap<&str, &str>,
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let asset =
            match github::find_asset(&self.release_information, artifact.artifact_name.as_str()) {
                Some(asset) => asset,
                None => return Err(Mss32AssetError::NotFound.into()),
            };

        if artifact.artifact_name.as_str() == "mss32.dll" {
            self.update_mss32(asset, hashes, progress_callback)?;
        }

        Ok(())
    }

    fn update_mss32(
        &self,
        asset: &github::AssetInformation,
        hashes: &HashMap<&str, &str>,
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let expected_hash = hashes
            .get(&asset.name.as_str())
            .ok_or(Mss32AssetError::HashNotFound)?;

        let module_path = module::get_path();
        let install_dir = module_path.parent().ok_or(Mss32AssetError::WriteFailure)?;

        let download_path = install_dir.join(&asset.name).with_extension("part");

        http::download_file(
            asset.url.as_str(),
            download_path.as_path(),
            progress_callback,
        )?;

        if sha1::digest(download_path.as_path())
            .map_or(true, |asset_hash| asset_hash != *expected_hash)
        {
            return Err(Mss32AssetError::IntegrityFailure.into());
        }

        let mss_path = install_dir.join(asset.name.as_str());
        let old_mss_path = mss_path.with_extension("old");
        std::fs::remove_file(&old_mss_path).ok();
        std::fs::rename(&mss_path, &old_mss_path)?;
        std::fs::rename(download_path, &mss_path).inspect_err(|_e| {
            std::fs::rename(&old_mss_path, &mss_path).ok();
        })?;
        Ok(())
    }
}

impl Component for Mss32Component {
    fn name(&self) -> &str {
        "Miles Loader"
    }

    fn get_updates(&self) -> anyhow::Result<Vec<Update>> {
        let updates = [self.get_module_update("mss32.dll", "Miles Loader")?]
            .into_iter()
            .flatten()
            .collect();

        Ok(updates)
    }

    fn update(
        &self,
        updates: &[Update],
        status_update: &dyn Fn(String),
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let hashes_str = github::fetch_hashes(&self.release_information)?;
        let hashes = github::parse_hashes(hashes_str.as_str());

        for update_artifact in updates {
            status_update(format!("Downloading {}...", update_artifact.display_name));
            self.update_artifact(update_artifact, &hashes, progress_callback)?;
        }

        Ok(())
    }
}

pub enum Mss32AssetError {
    NotFound,
    HashNotFound,
    IntegrityFailure,
    WriteFailure,
}

impl Mss32AssetError {
    fn message(&self) -> &str {
        match self {
            Self::NotFound => "Couldn't find Miles Loader asset",
            Self::HashNotFound => "Couldn't find Miles Loader asset hash",
            Self::IntegrityFailure => "Miles Loader asset integrity verification failed",
            Self::WriteFailure => "Failed to write Miles Loader",
        }
    }
}

impl Display for Mss32AssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for Mss32AssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for Mss32AssetError {}
