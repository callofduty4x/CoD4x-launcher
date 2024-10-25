use super::component::{Component, Update};
use crate::launcher::cod4x as cod4x_module;
use crate::launcher::filesystem as fs;
use crate::launcher::http;
use crate::launcher::sha1;
use crate::launcher::updater::github;
use semver::Version;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::sync::Arc;

pub struct CoD4xComponent {
    release_information: Arc<github::ReleaseInformation>,
}

impl CoD4xComponent {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            release_information: Arc::new(github::fetch_release_information(
                "callofduty4x/CoD4x_Client_pub",
            )?),
        })
    }

    fn get_module_update(
        &self,
        pattern: &str,
        display_name: &str,
    ) -> anyhow::Result<Option<Update>> {
        let mut upstream_tag = self.release_information.as_ref().tag_name.clone();
        if upstream_tag.matches('.').count() < 2 {
            upstream_tag.push_str(".0");
        }

        let upstream_version = Version::parse(upstream_tag.as_str());

        match cod4x_module::get_module_version() {
            // If we can't get the current version, always update
            Err(_) => Ok(Some(Update {
                display_name: display_name.to_string(),
                artifact_name: pattern.to_string(),
                current: None,
                upstream: upstream_version.unwrap_or(Version::new(0, 0, 0)),
                requires_elevate: false,
                requires_restart: false,
            })),
            // We got a valid current version, compare with upstream
            Ok(current) => {
                let upstream = upstream_version?;
                if upstream > current {
                    Ok(Some(Update {
                        display_name: display_name.to_string(),
                        artifact_name: pattern.to_string(),
                        current: Some(current),
                        upstream,
                        requires_elevate: false,
                        requires_restart: false,
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn get_cod4x_mod_update(
        &self,
        base_path: std::path::PathBuf,
        file_name: &str,
        display_name: &str,
    ) -> anyhow::Result<Option<Update>> {
        let artifact_path = base_path.join(file_name);
        if !artifact_path.exists() {
            Ok(Some(Update {
                display_name: display_name.to_string(),
                artifact_name: file_name.to_string(),
                current: None,
                upstream: Version::new(1, 0, 0),
                requires_elevate: false,
                requires_restart: false,
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
                None => return Err(CoD4xAssetError::NotFound.into()),
            };

        match artifact.artifact_name.as_str() {
            "^cod4x_([0-9]+).dll" => self.update_cod4x(asset, hashes, progress_callback)?,
            "jcod4x_00.iwd" => {
                self.update_mod_asset(fs::appdata_main_path()?, asset, hashes, progress_callback)?
            }
            "cod4x_ambfix.ff" => {
                self.update_mod_asset(fs::appdata_zone_path()?, asset, hashes, progress_callback)?
            }
            "cod4x_patch.ff" => {
                self.update_mod_asset(fs::appdata_zone_path()?, asset, hashes, progress_callback)?
            }
            "cod4x_patchv2.ff" => {
                self.update_mod_asset(fs::appdata_zone_path()?, asset, hashes, progress_callback)?
            }
            _ => {}
        };

        Ok(())
    }

    fn update_cod4x(
        &self,
        asset: &github::AssetInformation,
        hashes: &HashMap<&str, &str>,
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let expected_hash = hashes
            .get(&asset.name.as_str())
            .ok_or(CoD4xAssetError::HashNotFound)?;

        let savepath = fs::appdata_bin_path()?;
        let version_dir = std::path::Path::new(&asset.name)
            .file_stem()
            .ok_or(CoD4xAssetError::NameError)?;
        let destination_dir = savepath.join(version_dir);
        std::fs::create_dir_all(&destination_dir)?;

        let cod4x_path = destination_dir.join(&asset.name);
        let download_path = cod4x_path.with_extension("part");
        http::download_file(
            asset.url.as_str(),
            download_path.as_path(),
            progress_callback,
        )?;

        if sha1::digest(download_path.as_path())
            .map_or(true, |asset_hash| asset_hash != *expected_hash)
        {
            return Err(CoD4xAssetError::IntegrityFailure.into());
        }

        std::fs::rename(download_path, cod4x_path)?;
        Ok(())
    }

    fn update_mod_asset(
        &self,
        base_path: std::path::PathBuf,
        asset: &github::AssetInformation,
        hashes: &HashMap<&str, &str>,
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        let expected_hash = hashes
            .get(&asset.name.as_str())
            .ok_or(CoD4xAssetError::HashNotFound)?;

        std::fs::create_dir_all(&base_path)?;

        let file_path = base_path.join(&asset.name);
        let download_path = file_path.with_extension("part");
        http::download_file(
            asset.url.as_str(),
            download_path.as_path(),
            progress_callback,
        )?;

        if sha1::digest(download_path.as_path())
            .map_or(true, |asset_hash| asset_hash != *expected_hash)
        {
            return Err(CoD4xAssetError::IntegrityFailure.into());
        }

        std::fs::rename(download_path, file_path)?;
        Ok(())
    }
}

impl Component for CoD4xComponent {
    fn name(&self) -> &str {
        "CoD4x game"
    }

    fn get_updates(&self) -> anyhow::Result<Vec<Update>> {
        let updates = [
            self.get_module_update("^cod4x_([0-9]+).dll", "CoD4x DLL")?,
            self.get_cod4x_mod_update(fs::appdata_main_path()?, "jcod4x_00.iwd", "jcod4x")?,
            self.get_cod4x_mod_update(fs::appdata_zone_path()?, "cod4x_ambfix.ff", "ambfix")?,
            self.get_cod4x_mod_update(fs::appdata_zone_path()?, "cod4x_patch.ff", "patch")?,
            self.get_cod4x_mod_update(fs::appdata_zone_path()?, "cod4x_patchv2.ff", "patch v2")?,
        ]
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

pub enum CoD4xAssetError {
    NotFound,
    NameError,
    HashNotFound,
    IntegrityFailure,
}

impl CoD4xAssetError {
    fn message(&self) -> &str {
        match self {
            Self::NotFound => "Couldn't find CoD4x asset",
            Self::NameError => "Unexpected CoD4x asset name",
            Self::HashNotFound => "Couldn't find CoD4x asset hash",
            Self::IntegrityFailure => "CoD4x asset integrity verification failed",
        }
    }
}

impl Display for CoD4xAssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for CoD4xAssetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for CoD4xAssetError {}
