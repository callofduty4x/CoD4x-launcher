use super::filesystem as fs;
use super::http;
use super::module;
use super::process;
use super::sha1;
use super::zip;
use crate::launcher::updater::github;
use core::ffi::{c_char, c_void, CStr};
use libloading::{Library, Symbol};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub fn load_module(
    mss32importprocs: *mut *mut c_void,
    mss32importnames: *const *const c_char,
    mss32importcount: i32,
) -> Result<libloading::Library, Miles32LoadError> {
    let names = c_strings_to_slices(mss32importnames, mss32importcount);

    let miles32path = std::path::Path::new("miles32.dll");
    let module_path = module::get_path();
    let install_dir = module_path.parent();

    let full_miles32path = if let Some(install_dir) = install_dir {
        install_dir.join(miles32path)
    } else {
        std::path::PathBuf::from(miles32path)
    };

    unsafe {
        let lib = match Library::new(full_miles32path) {
            Ok(lib) => lib,
            Err(_) => return Err(Miles32LoadError::ModuleNotFound),
        };

        for (i, name) in names.iter().enumerate() {
            let element_ptr = mss32importprocs.add(i);
            let proc: Result<Symbol<*mut c_void>, _> = lib.get(name);

            let proc = match proc {
                Ok(proc) => proc.try_as_raw_ptr(),
                Err(_) => None,
            };

            if let Some(proc) = proc {
                *element_ptr = proc;
            } else {
                return Err(Miles32LoadError::MissingProcedure(
                    convert_bytes_to_string(name).unwrap_or("<Error>".to_string()),
                ));
            }
        }

        Ok(lib)
    }
}

pub fn replace_module() -> anyhow::Result<()> {
    let module_path = module::get_path();
    let install_dir = module_path
        .parent()
        .ok_or(ReplaceMiles32Error::InvalidPath)?;

    if !fs::is_writable(install_dir) {
        crate::launcher::msg_box::message_box(
            "CoD4x launcher needs to update file miles32.dll and will require elevated permissions",
            "Call of Duty 4 - Launcher",
        );
        process::restart(process::Privileges::Admin, Some("+set elevated 1"))?;
    }

    let savepath = fs::appdata_bin_path()?;
    let zip_miles32_path = savepath.join("miles32.zip");
    let org_miles32_path = savepath.join("miles32.dll");
    let new_miles32_path = install_dir.join("miles32.dll");
    std::fs::create_dir_all(&savepath)?;

    const MILES32_HASH: &str = "055dc05a4c175b84dffb87b2380714128e5b27dd";

    if sha1::digest(org_miles32_path.as_path())
        .map_or(true, |module_hash| module_hash != MILES32_HASH)
    {
        let release_info = github::fetch_release_information("callofduty4x/CoD4x_Client_pub")?;
        let mss_asset =
            github::find_asset(&release_info, "^mss$").ok_or(ReplaceMiles32Error::AssetNotFound)?;

        http::download_file(
            mss_asset.url.as_str(),
            zip_miles32_path.as_path(),
            &http::DummyProgress {},
        )?;

        zip::extract_file(
            zip_miles32_path.as_path(),
            std::path::Path::new("miles32.dll"),
            org_miles32_path.as_path(),
        )?;

        if sha1::digest(org_miles32_path.as_path())
            .map_or(true, |module_hash| module_hash != MILES32_HASH)
        {
            return Err(ReplaceMiles32Error::IntegrityFailure.into());
        } else {
            std::fs::remove_file(zip_miles32_path.as_path()).ok();
        }
    }

    std::fs::copy(org_miles32_path, new_miles32_path)?;

    Ok(())
}

fn c_strings_to_slices<'a>(ptr: *const *const c_char, count: i32) -> Vec<&'a [u8]> {
    let mut slices = Vec::new();

    unsafe {
        for i in 0..count {
            let c_str_ptr = *ptr.add(i as usize);
            if !c_str_ptr.is_null() {
                let c_str = CStr::from_ptr(c_str_ptr);
                slices.push(c_str.to_bytes());
            }
        }
    }

    slices
}

fn convert_bytes_to_string(bytes: &[u8]) -> Result<String, std::str::Utf8Error> {
    let string_slice = std::str::from_utf8(bytes)?;
    Ok(string_slice.to_string())
}

pub enum Miles32LoadError {
    ModuleNotFound,
    MissingProcedure(String),
}

impl Miles32LoadError {
    fn message(&self) -> String {
        match self {
            Self::ModuleNotFound => "Miles32 DLL not found".to_string(),
            Self::MissingProcedure(name) => format!("Missing Miles32 procedure '{}'", name),
        }
    }
}

impl Display for Miles32LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for Miles32LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for Miles32LoadError {}

enum ReplaceMiles32Error {
    InvalidPath,
    IntegrityFailure,
    AssetNotFound,
}

impl ReplaceMiles32Error {
    fn message(&self) -> &str {
        match self {
            Self::InvalidPath => "Invalid path",
            Self::IntegrityFailure => "Integrity verification failed",
            Self::AssetNotFound => "Couldn't find asset",
        }
    }
}

impl Display for ReplaceMiles32Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ReplaceMiles32Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ReplaceMiles32Error {}
