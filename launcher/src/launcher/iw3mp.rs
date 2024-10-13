use super::filesystem as fs;
use super::http;
use super::module;
use super::process;
use super::security_info;
use super::sha1;
use super::zip;
use crate::launcher::updater::github;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::{Read, Seek, SeekFrom, Write};
use winapi::um::winnt::{
    IMAGE_FILE_32BIT_MACHINE, IMAGE_FILE_EXECUTABLE_IMAGE, IMAGE_FILE_LARGE_ADDRESS_AWARE,
    IMAGE_FILE_RELOCS_STRIPPED,
};

pub fn is_pure() -> bool {
    let module: *const u8 = 0x401000 as *const u8;
    let module_text_end: *const u8 = 0x690429 as *const u8;
    let rdata: *const u8 = 0x691520 as *const u8;
    let rdata_end: *const u8 = 0x71b000 as *const u8;

    let text_len = unsafe { module_text_end.offset_from(module) as usize };
    let rdata_len = unsafe { rdata_end.offset_from(rdata) as usize };

    let text_checksum = adler32(module, text_len);
    if text_checksum != 0xD0D368F6 {
        return false;
    }

    let rdata_checksum = adler32(rdata, rdata_len);
    if rdata_checksum != 0xAA33BC12 {
        return false;
    }

    true
}

pub fn is_large_address_aware() -> bool {
    is_large_address_aware_impl().unwrap_or(true)
}

pub fn replace_module() -> anyhow::Result<()> {
    let module_path = module::get_path();
    let install_dir = module_path.parent().ok_or(ReplaceIw3mpError::InvalidPath)?;

    if !fs::is_writable(install_dir) {
        crate::launcher::msg_box::message_box(
            "CoD4x launcher needs to update file iw3mp.exe and will require elevated permissions",
            "Call of Duty 4 - Launcher",
        );
        process::restart(process::Privileges::Admin, Some("+set elevated 1"))?;
    }

    let savepath = fs::appdata_bin_path()?;
    let zip_iw3mp_path = savepath.join("iw3mp.zip");
    let save_iw3mp_path = savepath.join("iw3mp.exe");
    let org_iw3mp_path = install_dir.join("iw3mp.exe");
    let new_iw3mp_path = install_dir.join("iw3mp.new");
    std::fs::create_dir_all(&savepath)?;

    const IW3MP_HASH: &str = "3323b3882f9465a4c66cf298833435150effb153";

    if sha1::digest(save_iw3mp_path.as_path()).map_or(true, |module_hash| module_hash != IW3MP_HASH)
    {
        let release_info = github::fetch_release_information("callofduty4x/CoD4x_Client_pub")?;
        let core_asset =
            github::find_asset(&release_info, "^core$").ok_or(ReplaceIw3mpError::AssetNotFound)?;

        http::download_file(
            core_asset.url.as_str(),
            zip_iw3mp_path.as_path(),
            &http::DummyProgress {},
        )?;

        zip::extract_file(
            zip_iw3mp_path.as_path(),
            std::path::Path::new("iw3mp.exe"),
            save_iw3mp_path.as_path(),
        )?;

        if sha1::digest(save_iw3mp_path.as_path())
            .map_or(true, |module_hash| module_hash != IW3MP_HASH)
        {
            return Err(ReplaceIw3mpError::IntegrityFailure.into());
        } else {
            std::fs::remove_file(zip_iw3mp_path.as_path()).ok();
        }
    }

    let security_info = security_info::get(org_iw3mp_path.as_path());

    std::fs::copy(&save_iw3mp_path, &new_iw3mp_path)?;
    make_large_address_aware(new_iw3mp_path.as_path())?;
    self_replace::self_replace(new_iw3mp_path.as_path())?;

    if let Ok(info) = security_info {
        security_info::set(org_iw3mp_path.as_path(), &info)?;
    }

    Ok(())
}

pub fn startup() -> i32 {
    unsafe {
        __iw3mp_security_init_cookie();
        __iw3mp_tmainCRTStartup()
    }
}

fn is_large_address_aware_impl() -> std::io::Result<bool> {
    let mut file = std::fs::File::open(module::get_path())?;
    file.seek(SeekFrom::Start(286))?;

    let mut buffer = [0u8; 2];
    file.read_exact(&mut buffer)?;

    let flags = u16::from_le_bytes(buffer);
    Ok(flags & IMAGE_FILE_LARGE_ADDRESS_AWARE != 0)
}

fn make_large_address_aware(path: &std::path::Path) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(false)
        .open(path)?;

    const FLAGS: u16 = IMAGE_FILE_32BIT_MACHINE
        | IMAGE_FILE_EXECUTABLE_IMAGE
        | IMAGE_FILE_LARGE_ADDRESS_AWARE
        | IMAGE_FILE_RELOCS_STRIPPED;

    file.seek(SeekFrom::Start(286))?;
    file.write_all(&FLAGS.to_le_bytes())
}

fn adler32(data: *const u8, len: usize) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;

    const PRIME: u32 = 65521;
    for i in 0..len {
        a = unsafe { (a + *data.add(i) as u32) % PRIME };
        b = (b + a) % PRIME;
    }

    (b << 16) | a
}

unsafe fn __iw3mp_security_init_cookie() {
    type CdeclFn = unsafe extern "C" fn();
    let func: CdeclFn = std::mem::transmute(0x67f189_usize);
    func()
}

#[allow(non_snake_case)]
unsafe fn __iw3mp_tmainCRTStartup() -> i32 {
    type CdeclFn = unsafe extern "C" fn() -> i32;
    let func: CdeclFn = std::mem::transmute(0x67475c_usize);
    func()
}

enum ReplaceIw3mpError {
    InvalidPath,
    IntegrityFailure,
    AssetNotFound,
}

impl ReplaceIw3mpError {
    fn message(&self) -> &str {
        match self {
            Self::InvalidPath => "Invalid path",
            Self::IntegrityFailure => "Integrity verification failed",
            Self::AssetNotFound => "Couldn't find asset",
        }
    }
}

impl Display for ReplaceIw3mpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ReplaceIw3mpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ReplaceIw3mpError {}
