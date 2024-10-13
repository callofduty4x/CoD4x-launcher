use std::ffi::CString;
use user32::MessageBoxA;

pub fn message_box(message: &str, title: &str) {
    let lp_text = CString::new(message).unwrap();
    let lp_caption = CString::new(title).unwrap();
    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            0,
        );
    }
}
