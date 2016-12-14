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
        let syspath_ptr: LPSTR = mem::transmute::<_, LPSTR>(&mut syspath);
        let len = GetSystemDirectoryA(syspath_ptr, MAX_PATH as u32) as usize + 1;
        match CStr::from_bytes_with_nul(&syspath[..len+1]) {
            Ok(c) => c.to_owned(),
            Err(e) => {
                CString::new("C:\\WINDOWS\\system32").unwrap()
            }
        }
    }
}

unsafe fn write_bytes(s: &[u8], p: *mut c_char) {
    for (i, x) in s.iter().enumerate() {
        *p.offset(i as isize) = *x as i8;
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
        let procaddr = mem::transmute::<FARPROC, PFNDirectInput8Create>(GetProcAddress(hMod, fnName.as_ptr() as LPCSTR));
        writeln!(f, "function addr is {:p}", &procaddr).unwrap();
        let res = (procaddr)(inst, version, riid, out, u);

        // do our memory changes
        let addr1 = 0x0086ED7Cusize as *mut c_char;
        let addr2 = 0x008BD900usize as *mut c_char;
        let addr3 = 0x008BD93Cusize as *mut c_char;
        let addr4 = 0x008BD9C8usize as *mut c_char;
        let addr5 = 0x008BD9E4usize as *mut c_char;

        write_bytes(b"localhost\0", addr1);
        write_bytes(b"localhost\0", addr2);
        write_bytes(b"localhost\0", addr3);
        write_bytes(b"localhost\0", addr4);
        write_bytes(b"localhost\0", addr5);
        res
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(module: HMODULE, reason: DWORD, _reserved: LPVOID) -> BOOL {
    // https://msdn.microsoft.com/en-us/library/windows/desktop/ms682583(v=vs.85).aspx
    match reason {
        1 => {
            // DLL_PROCESS_ATTACH
            unsafe {
                DisableThreadLibraryCalls(module);
            }
        },
        0 => {
            // DLL_PROCESS_DETACH
        },
        _ => ()
    }
    1
}
