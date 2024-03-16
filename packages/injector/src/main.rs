use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};

use _utils::*;

fn main() {
    env_logger::init();

    let server = std::thread::spawn(|| {
        let mut conn = create_server("rust_winhook_demo".to_string()).unwrap();
        println!("[runtime] Named pipe server is running");

        loop {
            let msg: Msg = conn.read().unwrap();
            match msg {
                Msg::Log(s) => println!("[dll] {}", s),
                Msg::Terminated => {
                    println!("[dll] Terminated");
                    break;
                }
            }
        }
    });

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

        // find target process by name
        let target_process = OwnedProcess::find_first_by_name("rust-winhook-demo-core").unwrap();
        println!(
            "[runtime] target process PID: {}",
            target_process.pid().unwrap()
        );

        // create a new syringe for the target process
        let syringe = Syringe::for_process(target_process);

        // inject the payload into the target process
        let injected_payload = syringe
            .inject({
                if cfg!(target_arch = "x86") {
                    "target/i686-pc-windows-msvc/release/_dll.dll"
                } else {
                    "target/release/_dll.dll"
                }
            })
            .unwrap();
        if injected_payload.guess_is_loaded() {
            println!("[runtime] DLL injected successfully");
        } else {
            println!("[runtime] DLL injection failed");
        }
    })
    .join()
    .unwrap();

    server.join().unwrap();
}
