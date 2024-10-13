use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM};
use winapi::um::winnt::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};

pub fn get_error_string() -> String {
    unsafe {
        let code = GetLastError();
        let lang = MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD;
        let mut buf = [0; 1024];
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            std::ptr::null(),
            code,
            lang,
            buf.as_mut_ptr(),
            1024,
            std::ptr::null_mut(),
        );

        let end = buf.iter().position(|&i| i == 0).unwrap_or(1024);
        OsString::from_wide(&buf[..end])
            .into_string()
            .unwrap_or("Error while decoding system error message".to_string())
    }
}
