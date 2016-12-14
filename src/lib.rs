extern crate winapi;
extern crate kernel32;
extern crate libc;
//#[macro_use] extern crate minhook;

use std::mem;

use std::fs::File;
use std::io::Write;
use std::ffi::{CStr, CString};

use winapi::*;
use winapi::minwindef::*;
use winapi::guiddef::*;
use winapi::unknwnbase::*;
use kernel32::*;

#[allow(non_snake_case)]
//#[no_mangle]
#[export_name = "DirectInput8Create"]
pub extern "stdcall" fn DirectInput8Create(inst: HINSTANCE, version: DWORD, _riid: *const IID, _out: *mut LPVOID, _u: LPUNKNOWN) -> i32 {
    let mut f = File::create("foo.txt").unwrap();
    let a: Option<isize> = None;
    writeln!(f, "lol it opened {:?}\n", a).unwrap();

    let syspath = unsafe {
        let mut syspath: [u8; MAX_PATH] = [0; MAX_PATH];
        let syspath_ptr: LPSTR = mem::transmute(&mut syspath);
        GetSystemDirectoryA(syspath_ptr, MAX_PATH as u32);
        writeln!(f, "called GetSystemDirectoryA\n").unwrap();
        match CStr::from_bytes_with_nul(&syspath[..]) {
            Ok(c) => c.to_owned(),
            Err(e) => {
                writeln!(f, "error making cstring: {:?}\n", e).unwrap();
                CString::new("C:\\Windows\\system32\\").unwrap()
            }
        }
    };
    writeln!(f, "{}\n", syspath.to_string_lossy()).unwrap();
    1
}
