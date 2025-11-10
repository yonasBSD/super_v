// Standard Crates
#[allow(unused)]
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
use clap::{
    Parser,
    Subcommand
};

// My Crates
#[allow(unused)]
use super_v::{
    services::{
        clipboard_monitor::Monitor,
        clipboard_manager::Manager
    },
    common::{
        ClipboardItem,
        CmdIPC,
        IPCResponse
    },
    history::ClipboardHistory
};

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
    OpenGui
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

// ----------------------------- Main --------------------------------
fn main() {
    // History
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
        Command::Start => {
            println!("Starting service");
            let mut c_manager = match Manager::new() {
                Ok(manager) => {manager},
                Err(_) => {panic!("MANAGER COULD NOT BE STARTED!")},
            };
            c_manager._polling_service();
        },
        Command::OpenGui => println!("Opening GUI..."),
    }
}
// -------------------------------------------------------------------