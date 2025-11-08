use winapi::shared::ntdef::WCHAR;
use winapi::um::libloaderapi::GetModuleFileNameW;

pub unsafe fn is_iw3mp() -> bool {
    let mut buffer: [WCHAR; 1024] = [0; 1024];
    let len = unsafe {
        GetModuleFileNameW(
            core::ptr::null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
    } as usize;

    if len >= buffer.len() {
        return false;
    }

    let buffer = &buffer[0..len];
    let last_sep = buffer
        .iter()
        .rposition(|&ch| ch == b'/' as WCHAR || ch == b'\\' as WCHAR);
    if last_sep.is_none() {
        return false;
    }

    let cmp1 = *((0x6748ce) as *const u32);
    let cmp2 = *((0x6748d2) as *const u32);

    if cmp1 == 0x48c28bff && cmp2 == 0x74481774 {
        return false; //offical 1.0
    }

    if cmp1 == 0xe80875ff && cmp2 == 0xfffffeef {
        return false; //offical 1.1
    }

    if cmp1 == 0x89c53300 && cmp2 == 0x458bfc45 {
        return false; //offical 1.2
    }

    if cmp1 == 0x8c0f01fe && cmp2 == 0xffffff79 {
        return false; //offical 1.3
    }

    if cmp1 == 0x6a000072 && cmp2 == 0x1075ff01 {
        return false; //offical 1.4
    }

    if cmp1 == 0xc08510c4 && cmp2 == 0x7d8b0874 {
        return false; //offical 1.5
    }

    if cmp1 == 0xebe4458b && cmp2 == 0x40c03313 {
        return false; //offical 1.6
    }

    if cmp1 == 0xebe4458b && cmp2 == 0x40c03313 {
        return false; //offical 1.6
    }

    if cmp1 == 0xe9ffc883 && cmp2 == 0x552 {
        return true; //offical 1.8  steam
    }

    if cmp1 == 0xf02c7de8 && cmp2 == 0xe44589ff {
        return true; //offical 1.7
    }

    let filename = &buffer[last_sep.unwrap() + 1..];
    filename
        .iter()
        .take_while(|&&c| c != 0)
        .take(5)
        .copied()
        .eq("iw3mp".encode_utf16())
}
