// Standard Crates
use std::{
    process,
    thread,
    io::{
        Read,
        Write
    },
    os::unix::net::{
        UnixListener, 
        UnixStream
    },
    sync::atomic::{
        AtomicBool, 
        Ordering
    },
    sync::{
        Arc, 
        Mutex
    }
};

// External Crates
use arboard::{Clipboard};
use clap::{
    Parser,
    Subcommand
};

// My Crates
use super_v::services::clipboard_monitor::Monitor;
use super_v::history::ClipboardHistory;
use super_v::services::clipboard_manager::Manager;

/*
Notes:
- Add tests
- STOP USING UNWRAP
- Threaded Clipboard Manager that has a polling mechanism and manages history.
- Keyboard simulation using rdev (to paste when item is clicked)
- Mouse pointer monitoring to open the window at cursor location
- Emoji screen
*/

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the process
    Start,

    /// Open the GUI
    OpenGui,
}

#[derive(Parser, Debug)]
#[command(
    name = "super_v",
    version = "0.0.1",
    about = "Clipboard Service that looks like Win11",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

// const SOCKET_PATH: &str = "/tmp/superv.sock";

// fn run_client() {
//     // Connect to unix socket
//     match UnixStream::connect(SOCKET_PATH) {
//         Ok(mut stream) => {
//             // Send command to service
//         },
//         Err(_) => {
//             // Print error if cannot connect to stream
//             eprintln!("Error: Could not connect to super_v service.");
//             eprintln!("Is the service running?");
//         }
//     }
// }

// fn run_client(command: &str) {
//     // 1. Try to connect to the socket
//     match UnixStream::connect(SOCKET_PATH) {
//         Ok(mut stream) => {
//             // 2. Send the command as bytes
//             println!("Sending command to service: {}", command);
//             if let Err(e) = stream.write_all(command.as_bytes()) {
//                 eprintln!("Failed to write to socket: {}", e);
//                 return;
//             }

//             // 3. Wait for the "OK" response
//             // We use a small, fixed buffer to avoid deadlocks.
//             let mut response_buffer = [0; 32]; // 32 bytes is plenty for "OK"
            
//             match stream.read(&mut response_buffer) {
//                 Ok(bytes_read) => {
//                     // Convert only the bytes we read to a string
//                     let response = String::from_utf8_lossy(&response_buffer[..bytes_read]);
//                     println!("Service responded: {}", response.trim());
//                 }
//                 Err(e) => eprintln!("Error reading response: {}", e),
//             }
//         }
//         Err(_) => {
//             eprintln!("Error: Could not connect to SuperV service.");
//             eprintln!("Is the service running?");
//         }
//     }
// }

// fn run_service() -> Result<(), Box<dyn std::error::Error>> {
//     println!("Starting SuperV service...");

//     // 1. Clean up old socket (in case of a previous crash)
//     let _ = std::fs::remove_file(SOCKET_PATH);

//     // 2. Try to bind to the socket. This is our "lock".
//     let listener = match UnixListener::bind(SOCKET_PATH) {
//         Ok(listener) => listener,
//         Err(e) => {
//             eprintln!("Error: Could not bind to socket. Is SuperV already running?");
//             return Err(Box::new(e));
//         }
//     };

//     println!("Service bound to socket. Running... {}", SOCKET_PATH);

//     // --- We are now the official service. Start all background threads. ---

//     // 3. Create your shared data and stop signal
//     let history = Arc::new(Mutex::new(ClipboardHistory::new(50)));
//     let stop_signal = Arc::new(AtomicBool::new(false));


//     // 5. Spawn your poller thread
//     let poller_history = Arc::clone(&history);
//     let poller_stop = Arc::clone(&stop_signal);
//     thread::spawn(move || {
//         // This is your `poll` function!
//         poll(poller_stop, poller_history);
//     });

//     // 7. The main thread now just listens for IPC commands
//     println!("Waiting for commands. Press Ctrl+C to stop.");
//     for stream in listener.incoming() {
//         // Check if the stop signal has been set
//         if stop_signal.load(Ordering::SeqCst) {
//             println!("Stop signal received, shutting down service.");
//             break;
//         }

//         match stream {
//             Ok(mut stream) => {
//                 // Read the command from a client
//                 let mut command_buffer = [0; 128]; // 128 bytes for a command
//                 let bytes_read = match stream.read(&mut command_buffer) {
//                     Ok(n) => n,
//                     Err(_) => 0, // Connection error
//                 };

//                 if bytes_read == 0 {
//                     continue; // Empty command or error
//                 }
                
//                 let command = String::from_utf8_lossy(&command_buffer[..bytes_read]);
//                 let command = command.trim();

//                 println!("Received command: {}", command);

//                 // --- This is where you trigger actions ---
//                 if command == "open_gui" {
//                     println!("Told to open GUI!");
//                 }

//                 if command == "stop" {
//                     println!("Told to stop by client.");
//                     stop_signal.store(true, Ordering::SeqCst);
//                     // Write "OK" and break the loop to exit
//                     stream.write_all(b"OK").unwrap_or_default();
//                     break; 
//                 }
                
//                 // Write a response back to the client
//                 // We use .unwrap_or_default() to *ignore* a BrokenPipe panic
//                 // if the client hangs up before getting the response.
//                 stream.write_all(b"OK").unwrap_or_default();
//             }
//             Err(e) => {
//                 // Check stop signal again
//                 if stop_signal.load(Ordering::SeqCst) {
//                     break;
//                 }
//                 eprintln!("IPC stream error: {}", e);
//             }
//         }
//     }

//     println!("Service shutting down.");
//     let _ = std::fs::remove_file(SOCKET_PATH); // Final cleanup
//     Ok(())
// }

// ----------------------------- Main --------------------------------
fn main() {
    // // History
    // let mut ch = ClipboardHistory::new(20);
    // ch.add(ClipboardItem::Text("miaow".to_string()));
    // ch.add(ClipboardItem::Text("woof".to_string()));
    // ch.add(ClipboardItem::Text("rawr".to_string()));
    // ch.promote(1);
    // println!("{ch}");

    // // Monitor
    // let clipboard = Clipboard::new().unwrap();
    // clipboard.monitor();

    // // Clipboard Display
    // let mut clipboard = Clipboard::new().unwrap();
    // println!("{}", clipboard.get_item().unwrap());

    let args= Args::parse();
    match args.command {
        Command::Start => println!("Starting..."),
        Command::OpenGui => println!("Opening GUI..."),
    }
}
// -------------------------------------------------------------------