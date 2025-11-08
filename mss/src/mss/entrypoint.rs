use super::hook::{get_module_nt_header, patch_module};
use super::module_detect::is_iw3mp;
//use super::print::print;
//use crate::print;
//use crate::println;
use super::filesystem::get_cod4x_launcher_path;
use core::ffi::{c_char, c_void};
use core::ptr::addr_of_mut;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, HMODULE, TRUE};
use winapi::shared::ntdef::HANDLE;
use winapi::um::libloaderapi::{
    FreeLibrary, GetModuleHandleA, GetProcAddress, LoadLibraryA, LoadLibraryW,
};
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::winnt::{
    DLL_PROCESS_ATTACH, IMAGE_DOS_SIGNATURE, PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS32,
};
use winapi::um::winuser::MessageBoxA;

static mut OLDENTRYPOINTADDR: [u8; 5] = [0; 5];

unsafe fn load_miles32() -> bool {
    let miles32 = LoadLibraryA(c"miles32.dll".as_ptr());
    if miles32.is_null() {
        return false;
    }
    for (i, symbol) in super::symbols::MSS32_SYMBOLS.iter().enumerate() {
        let proc = GetProcAddress(miles32, symbol.as_ptr());
        if proc.is_null() {
            FreeLibrary(miles32);
            return false;
        }
        super::symbols::MSS32_PROCS[i] = proc as *mut c_void;
    }

    true
}

unsafe fn msgbox(error: &core::ffi::CStr) {
    MessageBoxA(
        core::ptr::null_mut(),
        error.as_ptr(),
        c"Call of Duty 4 - Modern Warfare - Fatal Error".as_ptr(),
        0x10,
    );
}

unsafe fn die(error: &core::ffi::CStr) {
    msgbox(error);
    ExitProcess(1);
}

unsafe fn set_jump(addr: u32, destination: *const ()) {
    let baddr = addr as *mut u8;
    let jmpwidth = (destination as isize - baddr as isize - 5) as u32;
    core::ptr::write(baddr, 0xe9);
    let baddr = baddr.add(1);
    core::ptr::write(baddr as *mut u32, jmpwidth);
}

unsafe fn write_launcher_entrypoint(entrypointaddr: *const u8) {
    OLDENTRYPOINTADDR[0] = *entrypointaddr.add(0);
    OLDENTRYPOINTADDR[1] = *entrypointaddr.add(1);
    OLDENTRYPOINTADDR[2] = *entrypointaddr.add(2);
    OLDENTRYPOINTADDR[3] = *entrypointaddr.add(3);
    OLDENTRYPOINTADDR[4] = *entrypointaddr.add(4);
    set_jump(entrypointaddr as u32, load_launcher_exec as *const ());
}

unsafe fn write_original_entrypoint(entrypointaddr: *mut u8) {
    *entrypointaddr.add(0) = OLDENTRYPOINTADDR[0];
    *entrypointaddr.add(1) = OLDENTRYPOINTADDR[1];
    *entrypointaddr.add(2) = OLDENTRYPOINTADDR[2];
    *entrypointaddr.add(3) = OLDENTRYPOINTADDR[3];
    *entrypointaddr.add(4) = OLDENTRYPOINTADDR[4];
}

unsafe fn restore_original_entrypoint(entrypoint: *mut u8) -> bool {
    if OLDENTRYPOINTADDR[0] == 0 {
        return false;
    }
    patch_module(|e| unsafe { write_original_entrypoint(e) }, entrypoint)
}

unsafe fn get_proc<T>(module: HMODULE, name: &core::ffi::CStr) -> Option<T> {
    let ptr = GetProcAddress(module, name.as_ptr());
    if ptr.is_null() {
        None
    } else {
        Some(core::mem::transmute_copy(&ptr))
    }
}

unsafe fn start_launcher() {
    let launcher_path = match get_cod4x_launcher_path() {
        Some(path) => path,
        None => {
            msgbox(c"Couldn't find CoD4x launcher DLL");
            return;
        }
    };

    let launcher = LoadLibraryW(launcher_path.as_ptr());
    if launcher.is_null() {
        return;
    }

    let mss32importnames: [*const c_char; super::symbols::MSS_SYMBOL_COUNT] =
        core::array::from_fn(|i| super::symbols::MSS32_SYMBOLS[i].as_ptr());

    type TStartLauncherFun =
        unsafe extern "C" fn(HINSTANCE, *mut *mut c_void, *const *const c_char, i32);
    if let Some(start_launcher_proc) = get_proc::<TStartLauncherFun>(launcher, c"StartLauncher") {
        start_launcher_proc(
            0x400000 as HINSTANCE,
            addr_of_mut!(super::symbols::MSS32_PROCS) as *mut *mut c_void,
            mss32importnames.as_ptr(),
            super::symbols::MSS_SYMBOL_COUNT as i32,
        ); // should never return
    } else {
        FreeLibrary(launcher);
    }
}

unsafe extern "C" fn load_launcher_exec() -> i32 {
    let base = GetModuleHandleA(core::ptr::null());
    if base.is_null() {
        return -1;
    }

    let dos_header = base as PIMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return -1;
    }
    let nt_header_offset = (*dos_header).e_lfanew as usize;
    let nt_header = &*(base.byte_add(nt_header_offset) as PIMAGE_NT_HEADERS32);

    let entrypoint =
        nt_header.OptionalHeader.ImageBase + nt_header.OptionalHeader.AddressOfEntryPoint;
    if entrypoint == 0 {
        return -1;
    }

    if !restore_original_entrypoint(entrypoint as *mut u8) {
        return -1;
    }

    start_launcher(); // ideally never returns

    if !load_miles32() {
        die(c"Failed to load miles32.dll");
        return 0;
    }

    type TEntrypoint = unsafe extern "C" fn();

    let entrypoint = core::mem::transmute_copy::<u32, TEntrypoint>(&entrypoint);
    entrypoint();
    0
}

unsafe fn hook_entrypoint_for_launcher() -> bool {
    let Some((_base, nt_header)) = get_module_nt_header() else {
        return false;
    };
    let nt_header = &*nt_header;

    let entrypoint =
        nt_header.OptionalHeader.ImageBase + nt_header.OptionalHeader.AddressOfEntryPoint;
    if entrypoint == 0 {
        return false;
    }

    patch_module(
        |e| unsafe { write_launcher_entrypoint(e) },
        entrypoint as *const u8,
    )
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(
    _hinstance: HANDLE,
    call_reason: DWORD,
    _lpv_reserved: &u32,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        if is_iw3mp() {
            if !hook_entrypoint_for_launcher() && !load_miles32() {
                die(c"Failed to load miles32.dll");
                return FALSE;
            }
        } else if !load_miles32() {
            die(c"Failed to load miles32.dll");
            return FALSE;
        }
    }
    TRUE
}
