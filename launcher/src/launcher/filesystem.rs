use super::wstring;
use winapi::shared::minwindef::DWORD;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processenv::SetCurrentDirectoryW;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::SetTokenInformation;
use winapi::um::winbase::SetDllDirectoryW;
use winapi::um::winnt::{TokenVirtualizationEnabled, HANDLE, MAXIMUM_ALLOWED};

pub fn is_writable(path: &std::path::Path) -> bool {
    let p = path.join(std::path::Path::new("testfile.tmp"));
    let res = std::fs::File::create(p.as_path()).is_ok();
    _ = std::fs::remove_file(p.as_path());
    res
}

pub fn disable_directory_virtualization() {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        let mut disable_value: DWORD = 0;
        let disable_value_ptr = &mut disable_value as *mut _ as *mut winapi::ctypes::c_void;

        if OpenProcessToken(GetCurrentProcess(), MAXIMUM_ALLOWED, &mut token) != 0 {
            SetTokenInformation(
                token,
                TokenVirtualizationEnabled,
                disable_value_ptr,
                std::mem::size_of::<DWORD>() as u32,
            );
            CloseHandle(token);
        }
    }
}

pub fn get_appdata_cod4_path() -> anyhow::Result<std::path::PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;
    use winapi::shared::minwindef::LPVOID;
    use winapi::shared::ntdef::LPWSTR;
    use winapi::shared::winerror::SUCCEEDED;
    use winapi::um::combaseapi::CoTaskMemFree;
    use winapi::um::knownfolders::FOLDERID_LocalAppData;
    use winapi::um::shlobj::{SHGetKnownFolderPath, KF_FLAG_CREATE};

    let app_data = unsafe {
        let mut res: LPWSTR = std::ptr::null_mut();
        let status = SHGetKnownFolderPath(
            &FOLDERID_LocalAppData,
            KF_FLAG_CREATE,
            std::ptr::null_mut(),
            &mut res,
        );

        if SUCCEEDED(status) && !res.is_null() {
            let len = (0..).take_while(|&i| *res.add(i) != 0).count();
            let wide_slice = std::slice::from_raw_parts(res, len);
            let path = OsString::from_wide(wide_slice).into_string();
            CoTaskMemFree(res as LPVOID);
            path.ok()
        } else {
            None
        }
    }
    .unwrap_or(std::env::var("LOCALAPPDATA")?);

    Ok(PathBuf::from(app_data).join("CallofDuty4MW"))
}

pub fn appdata_bin_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_appdata_cod4_path()?.join("bin"))
}

pub fn appdata_main_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_appdata_cod4_path()?.join("main"))
}

pub fn appdata_zone_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_appdata_cod4_path()?.join("zone"))
}

pub fn set_current_directory(path: &std::path::Path) {
    unsafe {
        SetCurrentDirectoryW(wstring::Wstring::new(path).into());
    }
}

pub fn set_dll_directory(path: &std::path::Path) {
    unsafe {
        SetDllDirectoryW(wstring::Wstring::new(path).into());
    }
}
