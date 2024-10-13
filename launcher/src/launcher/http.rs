use curl::easy::{Easy2, Handler, WriteError};
use std::io::Write;
use std::time::Duration;

pub trait Progress {
    fn progress(&self, _dltotal: f64, _dlnow: f64) -> bool {
        true
    }
}

pub struct ProgressCallback {
    callback: Box<dyn Fn(f64) -> bool + 'static>,
}

impl ProgressCallback {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(f64) -> bool + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl Progress for ProgressCallback {
    fn progress(&self, dltotal: f64, dlnow: f64) -> bool {
        let p = if dltotal > 0.0 {
            dlnow / dltotal * 100.0
        } else {
            0.0
        };

        self.callback.as_ref()(p)
    }
}

pub struct DummyProgress;
impl Progress for DummyProgress {}

struct FileCollector<'a, P> {
    file: std::fs::File,
    progress: &'a P,
}

impl<'a, P: Progress> FileCollector<'a, P> {
    pub fn new(file: std::fs::File, progress: &'a P) -> Self {
        Self { file, progress }
    }
}

impl<'a, P: Progress> Handler for FileCollector<'a, P> {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        if self.file.write_all(data).is_err() {
            Ok(0)
        } else {
            Ok(data.len())
        }
    }

    fn progress(&mut self, dltotal: f64, dlnow: f64, _ultotal: f64, _ulnow: f64) -> bool {
        self.progress.progress(dltotal, dlnow)
    }
}

pub fn download_file<P: Progress>(
    url: &str,
    path: &std::path::Path,
    progress: &P,
) -> anyhow::Result<()> {
    let easy = build_easy_get(
        url,
        FileCollector::new(std::fs::File::create(path)?, progress),
    )?;
    easy.perform()?;
    Ok(())
}

struct Collector {
    data: Vec<u8>,
}

impl Collector {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.data.extend_from_slice(data);
        Ok(data.len())
    }
}

pub fn download_str(url: &str, timeout: Option<Duration>) -> anyhow::Result<String> {
    let mut easy = build_easy_get(url, Collector::new())?;
    if let Some(timeout) = timeout {
        easy.timeout(timeout)?;
    }
    easy.perform()?;
    let handler = easy.get_ref();

    Ok(String::from_utf8(handler.data.clone())?)
}

fn build_easy_get<H: Handler>(url: &str, handler: H) -> Result<Easy2<H>, curl::Error> {
    let mut easy = Easy2::new(handler);
    easy.get(true)?;
    easy.follow_location(true)?;
    easy.url(url)?;
    // TODO: consider using a user agent designated for this cod4 launcher
    easy.useragent("curl/8.9.1")?;
    easy.progress(true)?;
    Ok(easy)
}
