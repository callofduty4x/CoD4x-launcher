use winapi::shared::minwindef::BOOL;
use winapi::shared::ntdef::HANDLE;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};

fn load_miles32() -> Result<(), Miles32LoadError> {
    unsafe {
        let miles32 = LoadLibraryA(c"miles32.dll".as_ptr());
        if miles32.is_null() {
            return Err(Miles32LoadError::LoadLibraryFailed);
        }
        for symbol in super::symbols::MSS32_SYMBOLS {
            let proc = GetProcAddress(miles32, symbol.as_ptr());
            if proc.is_null() {
                return Err(Miles32LoadError::MissingSymbol);
            }
        }
    }

    Ok(())
}

enum Miles32LoadError {
    LoadLibraryFailed,
    MissingSymbol,
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(_hinstance: HANDLE, call_reason: u32, _lpv_reserved: &u32) -> BOOL {
    match call_reason {
        _ => {
            if let Err(_) = load_miles32() {
                return 0 as BOOL;
            }
            return 1 as BOOL;
        }
    }
}
