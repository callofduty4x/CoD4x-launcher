#![no_std]

use core::panic::PanicInfo;
use winapi::um::processthreadsapi::ExitProcess;

mod mss;

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    unsafe {
        ExitProcess(1);
        loop {}
    }
}
