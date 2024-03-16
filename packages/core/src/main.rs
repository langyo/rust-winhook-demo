use windows::{core::*, Win32::System::LibraryLoader::LoadLibraryA};

fn main() {
    println!("[core] Waiting 1s for hook");
    std::thread::sleep(std::time::Duration::from_millis(1000));

    println!("[core] Loading kernel32.dll");
    unsafe {
        let _ = LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _));
    }
    println!("[core] Loaded kernel32.dll");
}
