use once_cell::sync::Lazy;
use std::ffi::CStr;

use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HMODULE,
        System::LibraryLoader::{GetProcAddress, LoadLibraryA},
    },
};

use crate::utils::ipc_println;

type HookFnType = extern "system" fn(PCSTR) -> HMODULE;

pub static hooker: Lazy<GenericDetour<HookFnType>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"LoadLibraryA\0".as_ptr() as _)) };
    let ori: HookFnType = unsafe { std::mem::transmute(address) };
    unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() }
});

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
    ipc_println(format!("our_LoadLibraryA lpFileName = {:?}", file_name));
    unsafe { hooker.disable().unwrap() };
    let ret_val = hooker.call(lpFileName);
    ipc_println(format!(
        "our_LoadLibraryA lpFileName = {:?} ret_val = {:#X}",
        file_name, ret_val.0
    ));
    unsafe { hooker.enable().unwrap() };
    ret_val
}
