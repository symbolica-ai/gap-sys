#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use anyhow::Result;
use std::ffi::{c_int, CStr, CString};
use std::fmt;
use std::ptr;
use std::sync::Once;

pub struct Gap;

pub struct GapGuard;

impl Drop for GapGuard {
    fn drop(&mut self) {
        unsafe {
            SYSGAP_Leave();
        }
    }
}

pub struct GapElement {
    obj: Obj,
}

impl fmt::Display for GapElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cstr = CStr::from_ptr(GAP_CSTR_STRING(self.obj));
            write!(f, "{}", cstr.to_string_lossy())
        }
    }
}

impl From<GapElement> for usize {
    fn from(val: GapElement) -> Self {
        let string = val.to_string();
        string.parse::<usize>().unwrap()
    }
}

impl Gap {
    pub fn init() -> &'static Gap {
        // Use a static ONCE and OPTIONAL to hold the singleton
        static mut OPTIONAL: Option<Gap> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let arg1 = CString::new("gap").unwrap();
                let arg2 = CString::new("-l").unwrap();
                let arg3 = CString::new("/usr/share/gap").unwrap();
                let arg4 = CString::new("-q").unwrap();
                let arg5 = CString::new("-E").unwrap();
                let arg6 = CString::new("--nointeract").unwrap();
                let arg7 = CString::new("-x").unwrap();
                let arg8 = CString::new("4096").unwrap();

                let mut c_args = vec![
                    arg1.into_raw(),
                    arg2.into_raw(),
                    arg3.into_raw(),
                    arg4.into_raw(),
                    arg5.into_raw(),
                    arg6.into_raw(),
                    arg7.into_raw(),
                    arg8.into_raw(),
                    ptr::null_mut(),
                ];

                GAP_Initialize(
                    c_args.len() as c_int - 1,
                    c_args.as_mut_ptr(),
                    None,
                    None,
                    1,
                );

                OPTIONAL = Some(Gap {});
            });
            OPTIONAL.as_ref().unwrap()
        }
    }

    pub fn eval(&self, cmd: &str) -> Result<GapElement> {
        let c_cmd = CString::new(cmd).unwrap();

        let _guard = GapGuard;

        unsafe {
            SYSGAP_Enter();

            let obj = GAP_EvalString(c_cmd.into_raw() as *const Char);
            let obj = GAP_ElmList(obj, 1);
            let success = GAP_ElmList(obj, 1);

            if success == GAP_True {
                let obj = GAP_ElmList(obj, 5);
                Ok(GapElement { obj })
            } else {
                Err(anyhow::anyhow!("Error evaluating command"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Due to a bug which I don't feel like fixing right now, tests can't run in parallel.
    // Also, CI won't have GAP installed, so we skip the tests.

    #[ignore]
    #[test]
    fn test_group() {
        let gap = Gap::init();
        let gap_element = gap.eval("Group((1,2,3),(1,2));").unwrap();
        assert_eq!(gap_element.to_string(), "Group([ (1,2,3), (1,2) ])");
    }

    #[ignore]
    #[test]
    fn test_direct_product() {
        let gap = Gap::init();
        gap.eval("a:=DirectProduct(SymmetricGroup(7), SymmetricGroup(7));")
            .unwrap();
        let order: usize = gap.eval("Order(a);").unwrap().into();
        assert_eq!(order, 25401600);
    }
}
