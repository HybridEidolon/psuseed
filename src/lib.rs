extern crate winapi;
extern crate kernel32;
extern crate psapi;
extern crate user32;
extern crate libc;
//#[macro_use] extern crate minhook;
extern crate toml;
extern crate rustc_serialize;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use std::mem;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{self, Write, Read};
use std::ffi::{CStr, CString, OsStr};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use winapi::*;
use winapi::minwindef::*;
use winapi::guiddef::*;
use winapi::unknwnbase::*;
use winapi::winerror::*;
use kernel32::*;
use psapi::*;
use user32::*;

mod config;
mod logger;

use logger::SimpleLogger;

use config::Config;

lazy_static! {
    static ref CONFIG: Mutex<Option<Config>> = Mutex::new(None);
}

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
    let syspath = get_system_directory().into_string().unwrap() + "\\dinput8.dll";
    unsafe {
        let hMod = LoadLibraryA(syspath.as_ptr() as LPCSTR);
        let fnName = CString::new("DirectInput8Create").unwrap();
        let procaddr = mem::transmute::<FARPROC, PFNDirectInput8Create>(GetProcAddress(hMod, fnName.as_ptr() as LPCSTR));
        let res = (procaddr)(inst, version, riid, out, u);

        let mut cfg = match CONFIG.lock() {
            Ok(c) => c,
            Err(_) => return res
        };

        if let Some(true) = cfg.as_ref().and_then(|v| v.borderless) {
            match find_main_window() {
                Some(hwnd) => {
                    info!("Found main window. hwnd={:x}", hwnd as usize);
                    thread::sleep(Duration::from_millis(500));
                    let mut style = GetWindowLongA(hwnd, GWL_STYLE);
                    style &= !(WS_CAPTION | WS_THICKFRAME) as LONG;
                    SetWindowLongA(hwnd, GWL_STYLE, style);
                    let mut style = GetWindowLongA(hwnd, GWL_EXSTYLE);
                    style &= !(WS_EX_DLGMODALFRAME | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE) as LONG;
                    SetWindowLongA(hwnd, GWL_EXSTYLE, style);

                    let ((x, y), (width, height)) = get_desktop_dimensions(hwnd);

                    SetWindowPos(hwnd, 0 as HWND,
                        x as c_int,
                        y as c_int,
                        width as c_int,
                        height as c_int,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
                },
                None => {
                    error!("Unable to find main window; border not set")
                }
            }
        }

        res
    }
}

fn get_desktop_dimensions(hwnd: HWND) -> ((i32, i32), (u32, u32)) {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut minfo = MONITORINFO {
            cbSize: mem::size_of::<MONITORINFO>() as DWORD,
            rcMonitor: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0
            },
            rcWork: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0
            },
            dwFlags: 0
        };

        GetMonitorInfoA(monitor, &mut minfo as *mut MONITORINFO);
        let x = minfo.rcMonitor.left as i32;
        let y = minfo.rcMonitor.top as i32;
        let x2 = minfo.rcMonitor.right as i32;
        let y2 = minfo.rcMonitor.bottom as i32;
        let width = (x2 - x) as u32;
        let height = (y2 - y) as u32;
        ((x, y), (width, height))
    }
}

fn get_base_address() -> usize {
    unsafe {
        let base = GetModuleHandleA(0 as LPCSTR) as usize;
        base
    }
}

fn find_main_window() -> Option<HWND> {
    unsafe {
        struct ProcData {
            pub pid: DWORD,
            pub handle: Option<HWND>
        }
        unsafe fn is_main_window(hwnd: HWND) -> bool {
            GetWindow(hwnd, GW_OWNER) == 0 as HWND && (IsWindowVisible(hwnd) > 0)
        }
        unsafe extern "system" fn enumproc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let mut pdata = &mut *(lparam as *mut ProcData);
            let mut r: DWORD = 0;
            GetWindowThreadProcessId(hwnd, &mut r as LPDWORD);
            if pdata.pid != r {
                return TRUE;
            }
            pdata.handle = Some(hwnd);
            FALSE
        }

        let mut my_proc_id = GetCurrentProcessId();
        let mut procdata = ProcData {
            pid: 0,
            handle: None
        };
        procdata.pid = my_proc_id;
        let cb = Some((enumproc as unsafe extern "system" fn(HWND, LPARAM) -> BOOL)) as WNDENUMPROC;
        EnumWindows(cb, (&mut procdata as *mut ProcData) as LPARAM);
        procdata.handle
    }
}

fn get_executable_name() -> Option<String> {
    unsafe {
        let my_process = GetCurrentProcess();
        let process_name = get_cstring_fun(256, move|buf| {
            GetModuleFileNameA(0 as HINSTANCE, buf, 256 as DWORD) as usize
        });
        let mut path = PathBuf::from(process_name);
        path.file_name().map(|p| p.to_string_lossy().into_owned())
    }
}

fn load_config() -> io::Result<Config> {
    let mut file = File::open("psuseed.toml")?;
    let mut cfgstr = String::new();
    file.read_to_string(&mut cfgstr)?;
    match toml::decode_str(&cfgstr) {
        Some(c) => Ok(c),
        None => Ok(Default::default())
    }
}

fn disable_minimap(base: usize) {
    unsafe {
        let mut ptr;
        // direction
        ptr = (base + 0x004D7480) as *mut u8;
        for i in 0..10 {
            *ptr.offset(i) = 0x00;
        }
        // radar_map
        ptr = (base + 0x004D7594) as *mut u8;
        for i in 0..10 {
            *ptr.offset(i) = 0x00;
        }
    }
}

fn init() {
    let logger = match SimpleLogger::new("psuseed.log") {
        Ok(l) => l,
        Err(_) => return
    };

    match log::set_logger(move |max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(logger)
    }) {
        Ok(_) => {},
        Err(_) => return
    }

    let apply_mem_patches = match get_executable_name() {
        Some(ref p) if p == "option.exe" => {
            info!("Disabling plugin for option.exe");
            return
        },
        Some(ref p) if p == "PSUC.exe" => {
            warn!("Some settings cannot be applied to compressed PSUC.exe, please use an uncompressed executable. Download UPX from https://upx.github.io/ and decompress PSUC.exe using the command line.");
            false
        }
        _ => true
    };

    let base = get_base_address();
    info!("Process base address: base={:x}", base);

    let config = match load_config() {
        Ok(c) => {
            info!("Config loaded. config={:?}", c);
            c
        },
        Err(e) => {
            error!("Unable to load config. {}", e);
            return
        }
    };

    unsafe {
        if apply_mem_patches {
            if let Some(ref host) = config.host {
                info!("Setting hosts. host={}", host);
                let addr1 = (0x0046ED7C + base) as *mut c_char;
                let addr2 = (0x004BD900 + base) as *mut c_char;
                let addr3 = (0x004BD93C + base) as *mut c_char;
                let addr4 = (0x004BD9C8 + base) as *mut c_char;
                let addr5 = (0x004BD9E4 + base) as *mut c_char;
                write_bytes(host.as_bytes(), addr1);
                write_bytes(host.as_bytes(), addr2);
                write_bytes(host.as_bytes(), addr3);
                write_bytes(host.as_bytes(), addr4);
                write_bytes(host.as_bytes(), addr5);
            }

            if let Some(ref port) = config.patch_port {
                info!("Setting patch port. port={}", port);
                let addr = (0x0049CDE4 + base) as *mut c_int;
                *addr = *port as c_int;
            }
            if let Some(ref port) = config.login_port {
                info!("Setting login port. port={}", port);
                let addr = (0x0058F690 + base) as *mut c_int;
                *addr = *port as c_int;
            }

            // 1280 width: 0x0086FDFC = 0x500 (u32)
            // 720 height: 0x0086FE00 = 0x2D0 (u32)
            if let Some(ref width) = config.width {
                info!("Setting window width. width={}", width);
                let addr = (0x0046FDFC + base) as *mut c_int;
                *addr = *width as c_int;
            }
            if let Some(ref height) = config.height {
                info!("Setting window height. height={}", height);
                let addr = (0x0046FE00 + base) as *mut c_int;
                *addr = *height as c_int;
            }

            if let Some(true) = config.disable_minimap {
                info!("Disabling minimap");
                disable_minimap(base);
            }
        }
        {
            let mut cfg_guard = CONFIG.lock().unwrap();
            *cfg_guard = Some(config);
        }
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
            init();
        },
        0 => {
            // DLL_PROCESS_DETACH
        },
        _ => ()
    }
    1
}
