use super::component::{Component, Update};
use crate::launcher::filesystem as fs;
use crate::launcher::http;
use crate::launcher::sha1;
use crate::launcher::updater::github;
use semver::Version;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::sync::Arc;

const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct LauncherComponent {
    release_information: Arc<github::ReleaseInformation>,
}

impl LauncherComponent {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            release_information: Arc::new(github::fetch_release_information(
                "callofduty4x/CoD4x-launcher",
            )?),
        })
    }

    fn get_module_update(
        &self,
        pattern: &str,
        display_name: &str,
    ) -> anyhow::Result<Option<Update>> {
        let upstream_version = Version::parse(self.release_information.as_ref().tag_name.as_str())?;
        let current_version = Version::parse(LAUNCHER_VERSION)?;

        if upstream_version > current_version {
            Ok(Some(Update {
                display_name: display_name.to_string(),
                artifact_name: pattern.to_string(),
                current: Some(current_version),
                upstream: upstream_version,
                requires_elevate: false,
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
                None => return Err(LauncherAssetError::NotFound.into()),
            };

        if artifact.artifact_name.as_str() == "launcher.dll" {
            self.update_launcher(asset, hashes, progress_callback)?;
        }

        Ok(())
    }

    fn update_launcher(
        &self,
        asset: &github::AssetInformation,
        hashes: &HashMap<&str, &str>,
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let expected_hash = hashes
            .get(&asset.name.as_str())
            .ok_or(LauncherAssetError::HashNotFound)?;

        let savepath = fs::appdata_bin_path()?;
        std::fs::create_dir_all(&savepath)?;
        let launcher_path = savepath.join(&asset.name);
        let download_path = launcher_path.with_extension("part");
        http::download_file(
            asset.url.as_str(),
            download_path.as_path(),
            progress_callback,
        )?;

        if sha1::digest(download_path.as_path())
            .map_or(true, |asset_hash| asset_hash != *expected_hash)
        {
            return Err(LauncherAssetError::IntegrityFailure.into());
        }

        let old_launcher_path = launcher_path.with_extension("old");
        std::fs::remove_file(&old_launcher_path).ok();
        std::fs::rename(&launcher_path, &old_launcher_path)?;
        std::fs::rename(download_path, &launcher_path)?;
        Ok(())
    }
}

impl Component for LauncherComponent {
    fn name(&self) -> &str {
        "CoD4x module"
    }

    fn get_updates(&self) -> anyhow::Result<Vec<Update>> {
        // TODO: add mss32.dll (consider as its own component)
        let updates = [self.get_module_update("launcher.dll", "Launcher DLL")?]
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

pub enum LauncherAssetError {
    NotFound,
    HashNotFound,
    IntegrityFailure,
}

impl LauncherAssetError {
    fn message(&self) -> &str {
        match self {
            Self::NotFound => "Couldn't find launcher asset",
            Self::HashNotFound => "Couldn't find launcher asset hash",
            Self::IntegrityFailure => "Launcher asset integrity verification failed",
        }
    }
}

impl Display for LauncherAssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for LauncherAssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for LauncherAssetError {}
