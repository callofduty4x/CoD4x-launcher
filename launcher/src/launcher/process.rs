use super::{error, module, wstring};

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use user32::AllowSetForegroundWindow;
use winapi::um::processthreadsapi::GetProcessId;
use winapi::um::shellapi::{
    ShellExecuteExW, LPSHELLEXECUTEINFOW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use winapi::um::winuser::SW_SHOWNORMAL;

pub enum Privileges {
    #[allow(dead_code)]
    User,
    Admin,
}

pub fn restart(privileges: Privileges, params: Option<&str>) -> anyhow::Result<()> {
    let method = match privileges {
        Privileges::User => "open",
        Privileges::Admin => "runas",
    };

    let module_path = module::get_path();
    let method = wstring::Wstring::new(method);
    let exefile = wstring::Wstring::new(module_path);

    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.lpVerb = method.as_ptr();
    sei.lpFile = exefile.as_ptr();
    sei.fMask = SEE_MASK_NOCLOSEPROCESS;
    sei.nShow = SW_SHOWNORMAL;
    let cmdline = params.map(wstring::Wstring::new);
    if let Some(cmdline) = cmdline {
        sei.lpParameters = cmdline.as_ptr();
    }

    if unsafe { ShellExecuteExW(&mut sei as LPSHELLEXECUTEINFOW) } == 0 {
        return Err(RestartProcessError::new(error::get_error_string()).into());
    }

    if sei.hProcess.is_null() {
        std::process::exit(0);
    }
    unsafe { AllowSetForegroundWindow(GetProcessId(sei.hProcess)) };

    Ok(())
}

struct RestartProcessError {
    message: String,
}

impl RestartProcessError {
    fn new(msg: String) -> Self {
        Self {
            message: format!("Failed to restart process: {msg}"),
        }
    }
}

impl Display for RestartProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl Debug for RestartProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl Error for RestartProcessError {}
