use windows::{core::*, Win32::System::LibraryLoader::LoadLibraryA};

fn main() {
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Loading kernel32.dll");
    unsafe {
        let _ = LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _));
    }
    println!("Loaded kernel32.dll");
}
