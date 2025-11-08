use winapi::shared::minwindef::{DWORD, HINSTANCE, LPVOID};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::{
    IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER, PAGE_EXECUTE_READ, PAGE_READWRITE,
    PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS32, PIMAGE_SECTION_HEADER,
};

pub unsafe fn get_module_nt_header() -> Option<(HINSTANCE, PIMAGE_NT_HEADERS32)> {
    let base = GetModuleHandleA(core::ptr::null());
    if base.is_null() {
        return None;
    }

    let dos_header = base as PIMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return None;
    }
    let nt_header_offset = (*dos_header).e_lfanew as usize;
    Some((base, base.byte_add(nt_header_offset) as PIMAGE_NT_HEADERS32))
}

pub unsafe fn patch_module<F, TArg>(patch_fun: F, arg: TArg) -> bool
where
    F: Fn(TArg),
{
    let Some((base, nt_header)) = get_module_nt_header() else {
        return false;
    };

    let mut section_base: LPVOID = core::ptr::null_mut();
    let mut section_size: usize = 0;
    let section_count = (*nt_header).FileHeader.NumberOfSections as usize;
    for i in 0..section_count {
        // 0x400200
        let section_header_ptr = nt_header.byte_add(
            core::mem::size_of::<IMAGE_NT_HEADERS>()
                + i * core::mem::size_of::<IMAGE_SECTION_HEADER>(),
        ) as PIMAGE_SECTION_HEADER;
        let section_header = &*section_header_ptr;

        let name = &section_header.Name[0..5];
        if name.eq(".text".as_bytes()) {
            section_base = base.byte_add(section_header.VirtualAddress as usize) as LPVOID;
            section_size = (*section_header.Misc.VirtualSize()) as usize;
            break;
        }
    }
    if section_size == 0 {
        return false;
    }

    let mut old_protect: DWORD = 0;
    VirtualProtect(section_base, section_size, PAGE_READWRITE, &mut old_protect);
    patch_fun(arg);
    VirtualProtect(
        section_base,
        section_size,
        PAGE_EXECUTE_READ,
        &mut old_protect,
    );
    true
}
