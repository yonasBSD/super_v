// Standard Crates
use std::{
    process
};

// External Crates
use clap::{
    Parser,
    Subcommand
};

// My Crates
use super_v::{
    services::{
        clipboard_manager::Manager,
        clipboard_ipc_server::{
            read_payload,
            send_payload,
            create_default_stream,
            CmdIPC,
            Payload
        }
    },
    common::{
        ClipboardItem
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
    OpenGui,

    Send
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

fn start_manager_daemon() {
    let mut c_manager = match Manager::new() {
        Ok(manager) => {
            println!("Starting service...");
            manager
        },
        Err(_) => {
            eprintln!("Another instance of Manager already running.");
            process::exit(0);
        },
    };

    c_manager.start_daemon();
}

// ----------------------------- Main --------------------------------
fn main() {
    // History
    let mut ch = ClipboardHistory::new(20);
    ch.add(ClipboardItem::Text("miaow".to_string()));
    ch.add(ClipboardItem::Text("woof".to_string()));
    ch.add(ClipboardItem::Text("rawr".to_string()));
    ch.promote(1);

    // Daemon
    let args= Args::parse();
    match args.command {
        Command::Start => {
            start_manager_daemon();
        },
        Command::OpenGui => println!("Opening GUI..."),
        Command::Send => {
            let mut stream = create_default_stream().unwrap();

            send_payload(&mut stream, Payload::Cmd(CmdIPC::Delete(1)));

            let resp = read_payload(&mut stream);
            println!("{:?}", resp);
        }
    }
}
// -------------------------------------------------------------------