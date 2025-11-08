use core::ffi::c_ulong;
use winapi::shared::ntdef::WCHAR;
use winapi::um::processenv::GetEnvironmentVariableW;

#[macro_export]
macro_rules! wide {
    ($lit:expr) => {{
        const WIDE: &[u16] = {
            const S: &str = $lit;
            // Encode UTF-8 literal as UTF-16 at compile time
            const UTF16: &[u16] = &{
                let mut buf = [0u16; $lit.len() + 1];
                let mut i = 0;
                let mut j = 0;
                while i < S.len() {
                    let ch = S.as_bytes()[i] as char;
                    buf[j] = ch as u16;
                    i += 1;
                    j += 1;
                }
                buf[j] = 0;
                buf
            };
            UTF16
        };
        WIDE
    }};
}

pub fn get_appdata_path() -> Option<([WCHAR; 1024], usize)> {
    use winapi::shared::minwindef::LPVOID;
    use winapi::shared::ntdef::LPWSTR;
    use winapi::shared::winerror::SUCCEEDED;
    use winapi::um::combaseapi::CoTaskMemFree;
    use winapi::um::knownfolders::FOLDERID_LocalAppData;
    use winapi::um::shlobj::{SHGetKnownFolderPath, KF_FLAG_CREATE};

    let mut buffer: [WCHAR; 1024] = [0; 1024];
    unsafe {
        let mut res: LPWSTR = core::ptr::null_mut();
        let status = SHGetKnownFolderPath(
            &FOLDERID_LocalAppData,
            KF_FLAG_CREATE,
            core::ptr::null_mut(),
            &mut res,
        );

        if SUCCEEDED(status) && !res.is_null() {
            let len = (0..buffer.len())
                .take_while(|&i| {
                    let c = *res.add(i);
                    buffer[i] = c;
                    c != 0
                })
                .count();
            CoTaskMemFree(res as LPVOID);

            if len < buffer.len() {
                return Some((buffer, len));
            }
        }

        let len = GetEnvironmentVariableW(
            wide!("LOCALAPPDATA").as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as c_ulong,
        );
        if len != 0 && len < buffer.len() as u32 {
            return Some((buffer, len as usize));
        }
    }
    None
}

pub fn get_cod4x_launcher_path() -> Option<[WCHAR; 1024]> {
    if let Some((mut appdata, len)) = get_appdata_path() {
        static LAUNCHER_PATH: &[u16] = wide!("\\CallofDuty4MW\\bin\\launcher.dll");
        if len + LAUNCHER_PATH.len() < appdata.len() {
            appdata[len..len + LAUNCHER_PATH.len()].copy_from_slice(LAUNCHER_PATH);
        }
        return Some(appdata);
    }
    None
}
