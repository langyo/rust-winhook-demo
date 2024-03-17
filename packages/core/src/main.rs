use std::io::Read as _;
use windows::{core::*, Win32::System::LibraryLoader::LoadLibraryA};

#[no_mangle]
#[inline(never)]
pub extern "C" fn add_42(x: i32) -> i32 {
    x + 42
}

fn main() {
    println!("[core] Waiting 1s for hook");
    std::thread::sleep(std::time::Duration::from_millis(1000));

    println!("[core] Loading kernel32.dll");
    unsafe {
        let _ = LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _));
    }
    println!("[core] Loaded kernel32.dll");

    println!("[core] Inline hooking add_42 test begin");
    println!("[core] add_42's address is 0x{:x}", add_42 as usize);

    let result = add_42(100);
    println!("[core] add_42(100) = {}", result);

    println!("[core] Inline hooking add_42 test end");

    println!("[core] Press any key to exit...");
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();
}
