// Standard Crates
#[allow(unused)]
use std::{
    fmt,
    collections::{
        HashMap, 
        VecDeque
    }, 
    error::Error, 
    thread::{
        self,
        sleep
    }, 
    time::Duration, 
    io::{
        stdin, 
        stdout, 
        Write
    },
    sync::{
        Arc, 
        Mutex,
        atomic::{
            AtomicBool, 
            Ordering
        }
    }
};

// External Crates
use arboard::{Clipboard};
use termion::{
    event::Key, 
    input::TermRead, 
    raw::IntoRawMode
};

// ---------------------------- Error --------------------------------
#[derive(Debug)]
#[allow(unused)]
// Error when you try to overwrite a Pos
enum ClipboardErr {
    ClipboardEmpty
}

// Displays for the Errors
impl fmt::Display for ClipboardErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardErr::ClipboardEmpty => {
                write!(f, "Clipboard is empty. Please add copy something before trying again.")
            }
        }
    }
}

// Implement the structs as Errors
impl Error for ClipboardErr {}
// // -------------------------------------------------------------------


// ------------------------- Clipboard Item -----------------------------
// Note: Using "C" for now.
// Consider using specific types from arboard later on.
#[allow(unused)]
#[derive(Debug, Clone, PartialEq)] // PartialEQ needed for comparision
enum ClipboardItem {
    Text(String),
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
// -------------------------------------------------------------------


// --------------------- Hist Implementation -------------------------
#[allow(unused)]
struct ClipboardHistory {
    history: VecDeque<ClipboardItem>,
    max_size: usize,
}

#[allow(unused)]
impl ClipboardHistory {
    fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    // Adds a new Clipboard item to history
    fn add(&mut self, item: ClipboardItem) {
        // Check for item duplicates
        if let Some(pos) = self.history.iter().position(|i| i == &item) {
            // It already exists. Promote it.
            self.promote(pos);
            return;
        }

        // Add to 0 (front)
        self.history.push_front(item);

        // Remove old items as size exceeds
        if self.history.len() > self.max_size {
            self.history.pop_back();
        }
    }

    // Given an index, it will push it to the TOP
    fn promote(&mut self, pos: usize) {
        // Remove item as 'pos'th index
        let promoted_item = self.history.remove(pos).unwrap();

        // Add it to history's TOP
        self.history.push_front(promoted_item);
    }

    // Returns all Items of the clipboard history
    fn get_items(&self) -> &VecDeque<ClipboardItem> {
        &self.history
    }

}

// Display for ClipboardHistory is now much simpler
impl fmt::Display for ClipboardHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printable = String::from("POS     | ITEM     ");
        printable += "\r\n---------------";
        
        // No sorting needed! Just iterate.
        for (pos, item) in self.history.iter().enumerate() {
            match item {
                ClipboardItem::Image { width, height, .. } => {
                    printable += &format!("\r\n{}       | Image ({}, {})     ", pos, width, height);
                },
                ClipboardItem::Text(string) => {
                    printable += &format!("\r\n{}       | {}     ", pos, string.to_string());
                }
            }
        }
        
        write!(f, "{printable}")
    }
}
// -------------------------------------------------------------------


// ------------------- Clipboard Implementations ---------------------
#[allow(unused)]
trait GetItem {
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardErr>;
}

impl GetItem for Clipboard {
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


// -------------------- Monitor, just for fun ------------------------
#[allow(unused)]
trait Monitor {
    fn monitor(self);
}

impl Monitor for Clipboard {
    // Usage:
    // let clipboard = Clipboard::new().unwrap();
    // clipboard.monitor();
    
    fn monitor(self) {        
        let stop = Arc::new(AtomicBool::new(false));

        let kb_stop = stop.clone();
        let cm_stop = stop.clone();
        

        let kb_handle = thread::spawn(move || {
            let stdin = stdin();
            let mut stdout = stdout().into_raw_mode().unwrap();

            write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
            stdout.flush().unwrap();

            write!(stdout, "Monitoring Clipboard. Press 'q' to exit. \r\n").unwrap();
            stdout.flush().unwrap();

            for c in stdin.keys() {
                if kb_stop.load(Ordering::SeqCst) {
                    break;
                }

                if let Ok(Key::Char('q')) | Ok(Key::Char('Q')) = c {
                    kb_stop.store(true, Ordering::SeqCst);
                    break;
                }
            }
        });
        
        let clipboard = Arc::new(Mutex::new(self));
        
        let cm_handle = thread::spawn(move || {
            let mut stdout = stdout().into_raw_mode().unwrap();

            let mut previous_content = clipboard.lock().unwrap().get_item().unwrap();
            
            while !cm_stop.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(100));

                if let Ok(content) = clipboard.lock().unwrap().get_item() {
                    if content != previous_content {
                        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
                        
                        write!(stdout, "Monitoring Clipboard. Press 'q' to exit. \r\n").unwrap();
                        stdout.flush().unwrap();

                        write!(stdout, "\n\nClipboard Change Detected:\r\n\n```\r\n{}\r\n```\r\n", content).unwrap();
                        stdout.flush().unwrap();
                        
                        previous_content = content;
                    }
                }
            }
        });

        kb_handle.join().unwrap();
        cm_handle.join().unwrap();

    }
}
// -------------------------------------------------------------------


// ----------------------------- Main --------------------------------
fn main() {
    // let mut ch = ClipboardHistory::new(20);
    // ch.add(ClipboardItem::Text("miaow".to_string()));
    // ch.add(ClipboardItem::Text("woof".to_string()));
    // ch.add(ClipboardItem::Text("rawr".to_string()));
    // ch.promote(1);
    // println!("{ch}");

    let clipboard = Clipboard::new().unwrap();
    clipboard.monitor(); // <- Consumes Clipboard. Do not use for polling

    // let mut clipboard = Clipboard::new().unwrap();
    // println!("{}", clipboard.get_item().unwrap());
}
// -------------------------------------------------------------------