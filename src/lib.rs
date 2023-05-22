use std::ffi::{c_int, CStr, CString};
use std::fmt;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Gap;

pub struct GapElement {
    obj: Obj,
}

impl fmt::Display for GapElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { SYSGAP_Enter() };
        let cstr = unsafe { CStr::from_ptr(GAP_CSTR_STRING(self.obj)) };
        unsafe { SYSGAP_Leave() };
        write!(f, "{}", cstr.to_string_lossy())
    }
}

impl From<GapElement> for usize {
    fn from(val: GapElement) -> Self {
        // Convert string to usize
        let string = val.to_string();
        string.parse::<usize>().unwrap()
    }
}

impl Gap {
    pub fn init() -> Self {
        let arg1 = CString::new("gap").unwrap();
        let arg2 = CString::new("-l").unwrap();
        let arg3 = CString::new("/usr/local/gap/share/gap").unwrap();
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

        unsafe {
            GAP_Initialize(
                c_args.len() as c_int - 1,
                c_args.as_mut_ptr(),
                None,
                None,
                1,
            );
        }

        Self {}
    }

    pub fn eval(&self, cmd: &str) -> Result<GapElement, &'static str> {
        unsafe { SYSGAP_Enter() };

        let c_cmd = CString::new(cmd).unwrap();
        let obj = unsafe { GAP_EvalString(c_cmd.into_raw() as *const Char) };

        let len = unsafe { GAP_LenList(obj) };
        let obj = unsafe { GAP_ElmList(obj, 1) };
        let success = unsafe { GAP_ElmList(obj, 1) };

        if success == unsafe { GAP_True } {
            let obj = unsafe { GAP_ElmList(obj, 5) };
            unsafe { SYSGAP_Leave() };
            Ok(GapElement { obj })
        } else {
            unsafe { SYSGAP_Leave() };
            Err("error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group() {
        let gap = Gap::init();
        let gap_element = gap.eval("Group((1,2,3),(1,2));").unwrap();
        assert_eq!(gap_element.to_string(), "Group([ (1,2,3), (1,2) ])");
    }

    #[test]
    fn test_direct_product() {
        let gap = Gap::init();
        gap.eval("a:=DirectProduct(SymmetricGroup(7), SymmetricGroup(7));")
            .unwrap();
        let order: usize = gap.eval("Order(a);").unwrap().into();
        assert_eq!(order, 25401600);
    }
}
