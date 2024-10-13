use crate::launcher::http;
use semver::Version;

pub struct Update {
    pub display_name: String,
    pub artifact_name: String,
    pub current: Option<Version>,
    pub upstream: Version,
    pub requires_elevate: bool,
}

pub trait Component: Send + Sync {
    fn name(&self) -> &str;

    fn get_updates(&self) -> anyhow::Result<Vec<Update>>;

    fn update(
        &self,
        updates: &[Update],
        status_report: &dyn Fn(String),
        progress_callback: &http::ProgressCallback,
    ) -> anyhow::Result<()>;
}

pub type ComponentUpdates = (Vec<Update>, Box<dyn Component>);
