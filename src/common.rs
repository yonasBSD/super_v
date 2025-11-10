// Standard Crates
#[allow(unused)]
use std::{
    fmt,
    error::Error
};

// External Crates
use arboard::Clipboard;
use serde::{
    Serialize, 
    Deserialize
};

// My Crates
use crate::history::ClipboardHistory;

// ---------------------------- Error --------------------------------
/// Error types for clipboard operations.
#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum ClipboardErr {
    /// Returned when attempting to access an empty clipboard
    ClipboardEmpty,

    /// Returned when attempting to spawn Manager but an instance is already running.
    ManagerMultiSpawn,
}

// Displays for the Errors
impl fmt::Display for ClipboardErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardErr::ClipboardEmpty => {
                write!(f, "Clipboard is empty. Please add copy something before trying again.")
            },
            ClipboardErr::ManagerMultiSpawn => {
                write!(f, "Another manager instance is already running")
            }
        }
    }
}

// Implement the structs as Errors
impl Error for ClipboardErr {}
// -------------------------------------------------------------------


// ----------------------- Clipboard Item ----------------------------
/// Represents an item that can be stored in the clipboard.
/// 
/// This enum supports both text and image data types, allowing the clipboard
/// to handle multiple content formats.
#[allow(unused)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ClipboardItem {
    /// Plain text content
    Text(String),
    
    /// Image content with dimensions and raw bytes
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>
    }
}

// Make the item printable
impl fmt::Display for ClipboardItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardItem::Text(s) => write!(f, "{}", s.replace('\n', "\r\n")),
            ClipboardItem::Image {width, height, ..} => write!(f, "[Image: {width}x{height}]")
        }
    }
}

/// Trait for retrieving clipboard content as a ClipboardItem.
/// 
/// This trait provides a unified interface for getting clipboard content,
/// automatically detecting whether the content is text or an image.
#[allow(unused)]
pub trait GetItem {
    /// Retrieves the current clipboard content.
    /// 
    /// # Returns
    /// 
    /// * `Ok(ClipboardItem)` - The clipboard content as either Text or Image
    /// * `Err(ClipboardErr::ClipboardEmpty)` - If the clipboard is empty
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardErr>;
}

impl GetItem for Clipboard {
    /// Implementation of GetItem for arboard's Clipboard.
    /// 
    /// Attempts to retrieve clipboard content in the following order:
    /// 1. Image data (if available)
    /// 2. Text data (if available)
    /// 3. Returns ClipboardEmpty error if neither is available
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardErr> {
        if let Ok(img_dat) = self.get_image() {
            Ok(ClipboardItem::Image { 
                width: img_dat.width, 
                height: img_dat.height, 
                bytes: img_dat.bytes.to_vec()
            })
        } else if let Ok(str_data) = self.get_text() {
            Ok(ClipboardItem::Text(str_data))
        } else {
            Err(ClipboardErr::ClipboardEmpty)
        }
    }
}
// -------------------------------------------------------------------


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
#[derive(Serialize, Deserialize)]
pub struct IPCResponse { 
    history_snapshot: ClipboardHistory,
    message: Option<String>
}
// -------------------------------------------------------------------