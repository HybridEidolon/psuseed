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
use winapi::winerror::*;
use kernel32::*;

fn get_system_directory() -> CString {
    unsafe {
        let mut syspath: [u8; MAX_PATH] = [0; MAX_PATH];
        let syspath_ptr: LPSTR = mem::transmute(&mut syspath);
        let len = GetSystemDirectoryA(syspath_ptr, MAX_PATH as u32) as usize + 1;
        match CStr::from_bytes_with_nul(&syspath[..len+1]) {
            Ok(c) => c.to_owned(),
            Err(e) => {
                CString::new("C:\\WINDOWS\\system32").unwrap()
            }
        }
    }
}

#[allow(non_snake_case)]
type PFNDirectInput8Create = extern "stdcall" fn(HINSTANCE, DWORD, *const IID, *mut LPVOID, LPUNKNOWN) -> HRESULT;

#[allow(non_snake_case)]
//#[no_mangle]
#[export_name = "DirectInput8Create"]
pub extern "stdcall" fn DirectInput8Create(inst: HINSTANCE, version: DWORD, riid: *const IID, out: *mut LPVOID, u: LPUNKNOWN) -> HRESULT {
    let mut f = File::create("foo.txt").unwrap();
    let a: Option<isize> = None;
    writeln!(f, "lol it opened {:?}", a).unwrap();

    let syspath = get_system_directory().into_string().unwrap() + "\\dinput8.dll";
    writeln!(f, "syspath is {}", syspath).unwrap();
    unsafe {
        let hMod = LoadLibraryA(syspath.as_ptr() as LPCSTR);
        let fnName = CString::new("DirectInput8Create").unwrap();
        let procaddr: PFNDirectInput8Create = mem::transmute(GetProcAddress(hMod, fnName.as_ptr() as LPCSTR));
        writeln!(f, "function addr is {:p}", &procaddr).unwrap();
        let res = (procaddr)(inst, version, riid, out, u);
        res
    }
}
