extern crate winapi;
//#[macro_use] extern crate minhook;

use std::fs::File;
use std::io::Write;

#[allow(non_snake_case)]
//#[no_mangle]
#[export_name = "DirectInput8Create"]
pub extern "stdcall" fn DirectInput8Create(_hinstance: i32, _version: i32, _riid: *const i32, _out: *mut *mut i32, _u: i32) -> i32 {
    let mut f = File::create("foo.txt").unwrap();
    let a: Option<isize> = None;
    writeln!(f, "lol it opened {:?}", a).unwrap();
    1
}
