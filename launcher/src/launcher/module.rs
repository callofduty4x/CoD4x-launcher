use winapi::shared::ntdef::WCHAR;
use winapi::um::libloaderapi::GetModuleFileNameW;

pub fn get_path() -> std::path::PathBuf {
    let mut buffer: Vec<WCHAR> = vec![0; 260];

    loop {
        let len = unsafe {
            GetModuleFileNameW(
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        } as usize;

        if len < buffer.len() {
            buffer.resize(len, 0);
            break;
        }

        buffer.resize(buffer.len() * 2, 0);
    }

    let path = String::from_utf16_lossy(&buffer);
    std::path::PathBuf::from(path)
}
