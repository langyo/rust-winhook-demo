use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};

fn main() {
    // Run the target process first
    std::thread::spawn(|| {
        std::process::Command::new({
            if cfg!(target_arch = "x86") {
                "target/i686-pc-windows-msvc/release/rust-winhook-demo-core.exe"
            } else {
                "target/release/rust-winhook-demo-core.exe"
            }
        })
        .spawn()
        .unwrap();
    });

    // find target process by name
    let target_process = OwnedProcess::find_first_by_name("rust-winhook-demo-core").unwrap();
    println!("target process PID: {}", target_process.pid().unwrap());

    // create a new syringe for the target process
    let syringe = Syringe::for_process(target_process);

    // inject the payload into the target process
    let injected_payload = syringe
        .inject({
            if cfg!(target_arch = "x86") {
                "target/i686-pc-windows-msvc/release/dll.dll"
            } else {
                "target/release/dll.dll"
            }
        })
        .unwrap();
    if injected_payload.guess_is_loaded() {
        println!("DLL injected successfully");
    } else {
        println!("DLL injection failed");
    }
}
