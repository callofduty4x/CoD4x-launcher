use std::ffi::OsStr;
//use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::os::windows::ffi::OsStrExt;

#[derive(Default)]
pub struct Wstring {
    str: Vec<u16>,
}

impl Wstring {
    pub fn new<S: AsRef<OsStr>>(s: S) -> Self {
        Self {
            str: s.as_ref().encode_wide().chain(Some(0)).collect(),
        }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.str.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.str.as_mut_ptr()
    }
}

impl<S: AsRef<OsStr>> From<S> for Wstring {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

impl From<Wstring> for *const u16 {
    fn from(s: Wstring) -> Self {
        s.as_ptr()
    }
}

impl From<Wstring> for *mut u16 {
    fn from(mut s: Wstring) -> Self {
        s.as_mut_ptr()
    }
}

// impl Display for Wstring {
//     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//         //write!(f, "{}", self.str)
//         write!(f, "unimplemented")
//     }
// }
// 
// impl Debug for Wstring {
//     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//         //write!(f, "{}", self.str)
//         write!(f, "unimplemented")
//     }
// }
