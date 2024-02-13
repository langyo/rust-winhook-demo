use anyhow::{Context, Result};
use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};

fn init_named_pipe() -> Result<()> {
    use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
    use std::io::{self, prelude::*, BufReader};

    fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
        match conn {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Incoming connection failed: {}", e);
                None
            }
        }
    }

    let name = "/tmp/rust_winhook_demo.sock";
    let listener = match LocalSocketListener::bind(name) {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            return Err(e.into());
        }
        x => x?,
    };
    println!("Server will running at {}", name);

    let mut buffer = String::with_capacity(128);
    for conn in listener.incoming().filter_map(handle_error) {
        let mut conn = BufReader::new(conn);
        println!("Incoming connection!");

        loop {
            conn.read_line(&mut buffer)
                .context("Socket receive failed")?;
            print!("IPC Client answered: {}", buffer);

            conn.get_mut().write_all(b"OK\n")?;

            if buffer == "stop\n" {
                // Automatically terminate the host progress after 1 second
                std::thread::sleep(std::time::Duration::from_secs(1));
                std::process::exit(0);
            }

            buffer.clear();
        }
    }
    Ok(())
}

fn main() {
    // Start the named pipe server
    let server = std::thread::spawn(|| {
        init_named_pipe().unwrap();
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
    });

    // Wait for the server to finish
    server.join().unwrap();
}
