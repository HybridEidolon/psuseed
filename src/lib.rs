extern crate winapi;
extern crate kernel32;
extern crate psapi;
extern crate libc;
//#[macro_use] extern crate minhook;
extern crate toml;
extern crate rustc_serialize;

use std::mem;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use std::ffi::{CStr, CString, OsStr};

use winapi::*;
use winapi::minwindef::*;
use winapi::guiddef::*;
use winapi::unknwnbase::*;
use winapi::winerror::*;
use kernel32::*;
use psapi::*;

mod config;

use config::Config;

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

fn get_cstring_fun<F>(len: usize, f: F) -> String where F: FnOnce(*mut c_char) -> usize {
    let mut buffer = vec![0u8; len].into_boxed_slice();
    let read = unsafe {
        let mut slice = &mut buffer[..];
        (f)(slice.as_mut_ptr() as *mut c_char)
    };
    let mut vec = buffer.into_vec();
    vec.truncate(read);
    CString::new(vec).map(|c| c.into_string().unwrap_or("error into_string".to_string())).unwrap_or("error".to_string())
}

#[allow(non_snake_case)]
type PFNDirectInput8Create = extern "stdcall" fn(HINSTANCE, DWORD, *const IID, *mut LPVOID, LPUNKNOWN) -> HRESULT;

#[allow(non_snake_case)]
//#[no_mangle]
#[export_name = "DirectInput8Create"]
pub extern "stdcall" fn DirectInput8Create(inst: HINSTANCE, version: DWORD, riid: *const IID, out: *mut LPVOID, u: LPUNKNOWN) -> HRESULT {
    let mut log = File::create("psuseed.log").unwrap();
    let syspath = get_system_directory().into_string().unwrap() + "\\dinput8.dll";
    unsafe {
        let hMod = LoadLibraryA(syspath.as_ptr() as LPCSTR);
        let fnName = CString::new("DirectInput8Create").unwrap();
        let procaddr = mem::transmute::<FARPROC, PFNDirectInput8Create>(GetProcAddress(hMod, fnName.as_ptr() as LPCSTR));
        let res = (procaddr)(inst, version, riid, out, u);


        let my_process = GetCurrentProcess();
        let process_name = get_cstring_fun(256, move|buf| {
            GetModuleFileNameA(0 as HINSTANCE, buf, 256 as DWORD) as usize
        });
        let mut path = PathBuf::from(process_name);
        if let Some(ref p) = path.file_name() {
            if p == &OsStr::new("option.exe") {
                return res;
            }
        }

        // read config
        let config: Config = match File::open("psuseed.toml") {
            Ok(mut f) => {
                writeln!(log, "Opened psuseed.toml").unwrap();
                let mut cfgstr = String::new();
                f.read_to_string(&mut cfgstr).unwrap();
                match toml::decode_str(&cfgstr) {
                    Some(c) => {
                        writeln!(log, "Parsed contents of psuseed.toml: {:?}", c).unwrap();
                        c
                    },
                    None => {
                        writeln!(log, "Failed to parse psuseed.toml").unwrap();
                        Default::default()
                    }
                }
            },
            Err(e) => {
                writeln!(log, "Unable to open psuseed.toml: {}", e).unwrap();
                Default::default()
            }
        };

        if let Some(ref host) = config.host {
            let addr1 = 0x0086ED7Cusize as *mut c_char;
            let addr2 = 0x008BD900usize as *mut c_char;
            let addr3 = 0x008BD93Cusize as *mut c_char;
            let addr4 = 0x008BD9C8usize as *mut c_char;
            let addr5 = 0x008BD9E4usize as *mut c_char;
            write_bytes(host.as_bytes(), addr1);
            write_bytes(host.as_bytes(), addr2);
            write_bytes(host.as_bytes(), addr3);
            write_bytes(host.as_bytes(), addr4);
            write_bytes(host.as_bytes(), addr5);
        }

        if let Some(ref port) = config.patch_port {
            let addr = 0x0089CDE4usize as *mut c_int;
            *addr = *port as c_int;
        }
        if let Some(ref port) = config.login_port {
            let addr = 0x0098F690usize as *mut c_int;
            *addr = *port as c_int;
        }

        // do our memory changes

        // patch port 0x0089CDE4
        // login port 0x0098F690

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
