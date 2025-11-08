use super::filesystem as fs;
use core::ffi::{c_char, CStr};
use libloading::Library;
use semver::Version;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::ntdef::LPSTR;
use winapi::um::processenv::GetCommandLineA;

pub fn run(hinstance: HINSTANCE, version: Option<&String>) -> anyhow::Result<()> {
    unsafe {
        let module = load_module(version)?;

        type TWinMain = unsafe extern "stdcall" fn(HINSTANCE, HINSTANCE, LPSTR, i32) -> i32;
        let win_main = module.get::<TWinMain>(b"WinMain@16\0");
        if let Ok(win_main) = win_main {
            std::process::exit(win_main(
                hinstance,
                std::ptr::null_mut(),
                GetCommandLineA(),
                0,
            ));
        }
    }

    Err(CoD4xLoadError::MissingEntrypoint.into())
}

pub fn get_module_version() -> anyhow::Result<Version> {
    unsafe {
        let module = load_module(None)?;
        type TGetCoD4xVersion = unsafe extern "C" fn() -> *const c_char;
        let get_cod4x_version = module.get::<TGetCoD4xVersion>(b"GetCoD4xVersion\0")?;

        let mut version_str = CStr::from_ptr(get_cod4x_version()).to_str()?.to_string();
        if version_str.matches('.').count() < 2 {
            version_str.push_str(".0");
        }

        Ok(Version::parse(version_str.as_str())?)
    }
}

fn load_module(version: Option<&String>) -> anyhow::Result<libloading::Library> {
    let cod4x_bin_dir = fs::appdata_bin_path()?;

    let version_dir: Option<std::path::PathBuf> = match version {
        Some(version) => Some(cod4x_bin_dir.join(format!("cod4x_{version}"))),
        None => {
            let cod4x_dirs = cod4x_bin_dir.join("cod4x_*\\");
            let cod4x_dirs = cod4x_dirs.to_str().ok_or(CoD4xLoadError::ModuleNotFound)?;

            glob::glob_with(
                cod4x_dirs,
                glob::MatchOptions {
                    case_sensitive: false,
                    ..glob::MatchOptions::new()
                },
            )?
            .flatten()
            .last()
        }
    };
    let version_dir = version_dir.ok_or(CoD4xLoadError::ModuleNotFound)?;

    let fullpath = version_dir
        .file_name()
        .map(|filename| version_dir.join(filename).with_extension("dll"))
        .ok_or(CoD4xLoadError::ModuleNotFound)?;

    fs::set_dll_directory(&version_dir);
    unsafe { Ok(Library::new(fullpath)?) }
}

enum CoD4xLoadError {
    ModuleNotFound,
    MissingEntrypoint,
}

impl CoD4xLoadError {
    fn message(&self) -> &str {
        match self {
            Self::ModuleNotFound => "CoD4x DLL not found",
            Self::MissingEntrypoint => "Missing entrypoint in CoD4x DLL",
        }
    }
}

impl Display for CoD4xLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for CoD4xLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for CoD4xLoadError {}
