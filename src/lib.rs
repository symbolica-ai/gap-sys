#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use anyhow::Result;
use std::borrow::Cow;
use std::ffi::{c_int, CStr, CString};
use std::fmt;
use std::ptr;
use std::sync::Once;

pub struct Gap {
    print_fn: Obj,
    output_stream: TypOutputFile,
    output_str_obj: Obj,
}

impl Drop for Gap {
    fn drop(&mut self) {
        unsafe {
            CloseOutput(&mut self.output_stream);
        }
    }
}

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

impl fmt::Pointer for GapElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.obj)
    }
}

unsafe fn hex_str_to_ptr(hex_str: &str) -> Result<Bag, std::num::ParseIntError> {
    let without_prefix = hex_str.trim_start_matches("0x");
    let addr = usize::from_str_radix(without_prefix, 16)?;
    Ok(addr as Bag)
}

// Implement from string for GapElement
// Convert the hex string into a *mut Bag
impl From<&str> for GapElement {
    fn from(s: &str) -> Self {
        GapElement {
            obj: unsafe { hex_str_to_ptr(s.trim()).unwrap() },
        }
    }
}

impl Gap {
    pub fn init() -> Gap {
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

        let _guard = GapGuard;

        unsafe {
            SYSGAP_Enter();
        }

        let output_text_str_operation = unsafe {
            let raw_ptr = CString::new("OutputTextString").unwrap().into_raw();
            let obj = ValGVar(GVarName(raw_ptr));
            let _ = CString::from_raw(raw_ptr);
            obj
        };

        let (mut output_stream, output_str_obj) = unsafe {
            let output_str_obj = NEW_STRING(0);
            let obj = DoOperation2Args(output_text_str_operation, output_str_obj, GAP_True);
            let mut output: TypOutputFile = std::mem::zeroed();
            assert_eq!(OpenOutputStream(&mut output, obj), 1);
            (output, output_str_obj)
        };

        let print_fn = unsafe {
            let raw_ptr = CString::new("Print").unwrap().into_raw();
            let obj = GAP_ValueGlobalVariable(raw_ptr);
            let _ = CString::from_raw(raw_ptr);
            obj
        };

        Gap {
            print_fn,
            output_stream,
            output_str_obj,
        }
    }

    pub fn eval(&self, cmd: &str) -> Result<GapElement> {
        let c_cmd = CString::new(cmd).unwrap();

        let _guard = GapGuard;

        unsafe {
            SYSGAP_Enter();

            // Create a raw pointer to the CString, needs to be freed later
            let raw_ptr = c_cmd.into_raw();
            let obj = GAP_EvalString(raw_ptr as *const Char);
            // Drop the CString so it doesn't leak
            let _ = CString::from_raw(raw_ptr);

            let obj = GAP_ElmList(obj, 1);
            let success = GAP_ElmList(obj, 1);

            if success == GAP_True {
                let obj = GAP_ElmList(obj, 2);
                Ok(GapElement { obj })
            } else {
                Err(anyhow::anyhow!("Error evaluating command"))
            }
        }
    }

    pub fn elem_string(&mut self, element: &GapElement) -> String {
        let _guard = GapGuard;

        unsafe {
            SYSGAP_Enter();
        }

        unsafe {
            GAP_CallFunc1Args(self.print_fn, element.obj);
        }

        // Flush
        unsafe {
            Pr(b"\x03".as_ptr() as *const Char, 0, 0);
        }

        let cstr = unsafe { CStr::from_ptr(GAP_CSTR_STRING(self.output_str_obj)) };

        let copy = cstr.to_string_lossy().to_string();

        unsafe {
            SET_LEN_STRING(self.output_str_obj, 0);
        }

        copy
    }

    pub fn get_list_elem(&self, list: &GapElement, idx: usize) -> Result<GapElement> {
        let _guard = GapGuard;

        unsafe {
            SYSGAP_Enter();

            let obj = GAP_ElmList(list.obj, idx);
            Ok(GapElement { obj })
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
        let order: usize = gap.eval("Order(a);").unwrap().to_string().parse().unwrap();
        assert_eq!(order, 25401600);
    }

    #[ignore]
    #[test]
    fn test_nested_list() {
        let mut gap = Gap::init();
        let outer_list = gap.eval("[[1, 2, 3], [4, 5, 6]];;").unwrap();
        let inner_list = gap.get_list_elem(&outer_list, 1).unwrap();
        let element = gap.get_list_elem(&inner_list, 1).unwrap();
        let string = gap.elem_string(&element);
        assert_eq!(string, "1");
    }

    #[ignore]
    #[test]
    fn test_echo() {
        let mut gap = Gap::init();
        let hello = gap.eval("\"Hello, world!\";").unwrap();
        let string = gap.elem_string(&hello);
        println!("{}", string);
    }
}
