use super::wstring;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::accctrl::SE_FILE_OBJECT;
use winapi::um::aclapi::{GetNamedSecurityInfoW, SetNamedSecurityInfoW};
use winapi::um::winbase::LocalFree;
use winapi::um::winnt::{DACL_SECURITY_INFORMATION, PACL, PSECURITY_DESCRIPTOR};

pub struct SecurityDescriptor {
    acl: PACL,
    psd: PSECURITY_DESCRIPTOR,
}

impl SecurityDescriptor {
    pub fn new() -> Self {
        Self {
            acl: std::ptr::null_mut(),
            psd: std::ptr::null_mut(),
        }
    }
}

pub fn get(path: &std::path::Path) -> Result<SecurityDescriptor, SecurityDescriptorError> {
    unsafe {
        let mut descriptor = SecurityDescriptor::new();
        match GetNamedSecurityInfoW(
            wstring::Wstring::new(path).into(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut descriptor.acl,
            std::ptr::null_mut(),
            &mut descriptor.psd,
        ) {
            ERROR_SUCCESS => Ok(descriptor),
            code => Err(SecurityDescriptorError::GetError(code)),
        }
    }
}

pub fn set(
    path: &std::path::Path,
    descriptor: &SecurityDescriptor,
) -> Result<(), SecurityDescriptorError> {
    unsafe {
        match SetNamedSecurityInfoW(
            wstring::Wstring::new(path).into(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            descriptor.acl,
            std::ptr::null_mut(),
        ) {
            ERROR_SUCCESS => Ok(()),
            code => Err(SecurityDescriptorError::SetError(code)),
        }
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.psd);
        }
    }
}

pub enum SecurityDescriptorError {
    GetError(u32),
    SetError(u32),
}

impl SecurityDescriptorError {
    fn message(&self) -> String {
        match self {
            Self::GetError(code) => format!("Failed to get security descriptor: {}", code),
            Self::SetError(code) => format!("Failed to set security descriptor: {}", code),
        }
    }
}

impl Display for SecurityDescriptorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for SecurityDescriptorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for SecurityDescriptorError {}
