use super::component::{Component, Update};
use crate::launcher::http;
use crate::launcher::updater::github;
use semver::Version;
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
            }))
        } else {
            Ok(None)
        }
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
        _updates: &[Update],
        _status_report: &dyn Fn(String),
        _progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
