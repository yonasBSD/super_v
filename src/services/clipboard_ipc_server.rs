// System Crates
use std::{
    os::unix::net::{
        UnixListener, 
        UnixStream
    },
    io::{
        Write,
        Read
    },
    fs::remove_file
};

// External Crates
use serde::{
    Serialize, 
    Deserialize
};
use rmp_serde::{Serializer};

// My Crates
use crate::{common::{IPCServerError}, history::ClipboardHistory};

// ------------------------- IPC Items -------------------------------
/// Represents the commands that IPC Supports
/// 
/// This enum allows for the following commands:
/// * **Promote(usize)** - Command that promotes and item to top of history.
/// * **Delete(usize)** - Command that deletes an item from history given its pos.
/// * **Snapshot** - Command that retrieves the snapshot of the current Clipboard History
/// * **Clear** - Command that clears the entire clipboard History.
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub enum CmdIPC {
    Promote(usize),
    Delete(usize),
    Snapshot,
    Clear,
}

/// A data structure representing the Response of IPC.
/// 
/// Contains:
/// * **history_snapshot** - A snapshot of the current ClipboardHistory from the Clipboard Manager Daemon
/// * **message** - Optional message.
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct IPCResponse { 
    history_snapshot: ClipboardHistory,
    message: Option<String>
}

pub struct PayloadData {
    buf: Vec<u8>,
    len: [u8; 4]
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    Cmd(CmdIPC),
    Resp(IPCResponse),
}

impl Payload {
    fn to_payload(&self) -> PayloadData {
        let mut  buf: Vec<u8> = Vec::new();
        let _ = self.serialize(&mut Serializer::new(&mut buf)).ok();
        let payload_len: [u8; 4] = (buf.len() as u32).to_be_bytes();

        PayloadData { 
            buf: buf,
            len: payload_len
        }
    }
}
// -------------------------------------------------------------------

const SOCKET_PATH: &str = "/tmp/super_v.sock";

pub fn start() -> Result<UnixListener, IPCServerError> {
    // Create a new listener
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => {
            // Remove old sock file
            let _ = remove_file(SOCKET_PATH);
            listener
        },
        Err(err) => {
            return Err(IPCServerError::BindError(String::from(format!("{:?}", err))));
        }
    };

    // Return Listener
    Ok(listener)
}

pub fn default_stream() -> Result<UnixStream, IPCServerError> {
    match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => {
            Ok(stream)
        },
        Err(err) => {
            Err(IPCServerError::ConnectionError(format!("{:?}", err)))
        }
    }
}

pub fn send_payload(mut stream: UnixStream, item: Payload) {
    // Serialize command
    let payload = item.to_payload();

    // Send len
    // We know the size of the lenght (4).
    // Using that, we can extract the lenght of actual message (x)
    // and read for that len. 
    // This way sending message of changing length works.
    stream.write_all(&payload.len).unwrap();
    
    // Send data
    stream.write_all(&payload.buf).unwrap();

    // Ensure all buffer is written
    stream.flush().unwrap();
}

pub fn read_payload(mut stream: UnixStream) -> Payload {
    // Read length of message (u32)
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).unwrap();
    let req_len = u32::from_be_bytes(len_buf) as usize;

    // Read payload
    let mut payload = vec![0u8; req_len];
    stream.read_exact(&mut payload).unwrap();

    // deserialize
    let cmd: Payload = rmp_serde::from_slice(&payload).expect("failed to deserialize");
    cmd
}