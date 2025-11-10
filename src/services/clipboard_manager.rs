// System Crates
use std::{
    sync::{
        Arc, 
        Mutex, 
        atomic::{ 
            AtomicBool, 
            Ordering 
        }
    }, 
    thread::{self, JoinHandle, sleep}, 
    time::Duration
};

// External Crates
use arboard::Clipboard;

// My Crates
use crate::{
    common::{ 
        ClipboardItem, 
        GetItem 
    },
    history::ClipboardHistory
};

// Clipboard Size
const CLIPBOARD_SIZE: usize = 25;

/// # Manager
///  Holds shared services and thread handles for the clipboard manager.
///
/// Fields:
/// - _clipboard_service: Arc-wrapped clipboard service used to read the system clipboard.
/// - _shared_history: Arc-wrapped ClipboardHistory shared between threads.
/// - _stop_signal: Atomic flag used to request worker threads to stop.
/// - _polling_handle: Optional JoinHandle for the polling thread.
/// - _command_handle: Optional JoinHandle for the command-handling thread.
///
/// These fields are internal to the implementation and not intended for public API use.
/// Check implementation of Manager for usage.
pub struct Manager {
    pub _clipboard_service: Arc<Mutex<Clipboard>>,
    pub _shared_history: Arc<Mutex<ClipboardHistory>>,
    pub _stop_signal:Arc<AtomicBool>,

    pub _polling_handle: Option<JoinHandle<()>>,
    pub _command_handle: Option<JoinHandle<()>>,
}

impl Manager {
    
    /// Create a new Manager instance and configure global handlers.
    ///
    /// **Behavior**:
    /// - Allocates a ClipboardHistory with a fixed capacity.
    /// - Creates and wraps a Clipboard service in an Arc<Mutex<...>>.
    /// - Creates an Arc<AtomicBool> stop signal used by worker threads.
    /// - Installs a ctrl-c handler that updates the stop signal.
    ///
    /// **Panics / errors**:
    /// - This constructor unwraps the clipboard creation and will panic if the clipboard cannot be initialized.
    ///
    /// **Returns**:
    /// - A fully constructed Manager with no active thread handles.
    pub fn new() -> Self {
        // New history
        let history: Arc<Mutex<ClipboardHistory>> = Arc::new(
            Mutex::new(
                ClipboardHistory::new(CLIPBOARD_SIZE)
            )
        );

        // Clipboard service
        let _clipboard_service: Arc<Mutex<Clipboard>> = Arc::new(
            Mutex::new(
                Clipboard::new().unwrap()
            )
        );

        // Stop signal
        let _stop_signal: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        
        // Setup ctrl+c
        let ss_clone = _stop_signal.clone();
        let _ = ctrlc::set_handler(move || {
            ss_clone.store(false, Ordering::SeqCst);
        });

        // Return the manager object
        Self {
            _clipboard_service: _clipboard_service,
            _shared_history: history,
            _stop_signal: _stop_signal,

            // No handles yet.
            _polling_handle: None,
            _command_handle: None
        }
    }

    /// Start the polling service in a new background thread.
    ///
    /// **Behavior**:
    /// - Returns early with a log if a polling thread is already running.
    /// - Clones required Arcs for use inside the spawned thread.
    /// - The thread repeatedly:
    ///     * Sleeps for a fixed interval (500 ms).
    ///     * Attempts to read the current clipboard item (falling back to an empty item on error).
    ///     * Compares it with the last seen item and, if different, attempts to push it into ClipboardHistory.
    /// - Uses try_lock on locks to avoid blocking other threads; if a lock is unavailable it skips that iteration.
    /// - Exits when the stop signal is set.
    ///
    /// **Notes**:
    /// - This function stores the JoinHandle in _polling_handle.
    /// - Designed to be safe for use alongside a command thread that may also access shared_history.
    pub fn _polling_service(&mut self) {
        // Check if polling thread is already started
        let None = self._polling_handle else {
            eprintln!("Polling service is already running");
            return;
        };

        // Create clones of the Arc items needed.
        let clipboard_service = self._clipboard_service.clone();
        let stop_signal = self._stop_signal.clone();
        let shared_history = self._shared_history.clone();

        // Start the polling in a thread and store the handle
        self._polling_handle = Some(thread::spawn(move || {
            let empty_item = ClipboardItem::Text("".to_string());

            // Get the current item in clipboard. This will be compared with and edited
            let mut last_item = match clipboard_service.try_lock() {
                Ok(mut unlocked_clipboard) => {
                    match unlocked_clipboard.get_item() {
                        Ok(item) => {item},
                        Err(_) => {empty_item.clone()},
                    }
                },
                Err(_) => {empty_item.clone()},
            };
            
            while !stop_signal.load(Ordering::SeqCst) {
                // Poll every 500ms
                sleep(Duration::from_millis(500));

                // Item Checking
                let current_item = match clipboard_service.try_lock() {
                    Ok(mut unlocked_clipboard) => {
                        match unlocked_clipboard.get_item() {
                            Ok(item) => {item},
                            Err(_) => {empty_item.clone()},
                        }
                    },
                    Err(_) => {empty_item.clone()},
                };

                // This should be fine since _polling_service and _command_service both exist in the same process.
                // So no need for thread-to-thread communication management and can purely focus on IPC management.
                // Checks if item is new or not.
                if current_item != last_item {
                    // Acquire Lock
                    match shared_history.try_lock() {
                        Ok(mut unlocked_history) => {
                            // Add item to history
                            unlocked_history.add(current_item.clone());

                            // Update Last item
                            last_item = current_item
                        },
                        Err(_) => {/* Failed To Get Lock, Skip */},
                    }
                }
            }
        }));
    }

    // Starts the command service in a thread
    pub fn _command_service(&mut self) {

    }

    /// Start all configured background services.
    ///
    /// **Behavior**:
    /// - Calls _polling_service to start the clipboard poller.
    /// - Calls _command_service to start command handling.
    /// - Each service checks whether it is already running and will not start duplicate
    pub fn start_services(&mut self) {
        // Start the polling service
        self._polling_service();

        // Start the command service
        self._command_service();
    }

    /// Request shutdown and join worker threads.
    ///
    /// **Behavior**:
    /// - Sets the stop signal to request all worker threads to exit.
    /// - Takes ownership of the stored thread handles and attempts to join them.
    /// - Joining is performed from a short-lived helper thread to avoid blocking the caller.
    ///
    /// **Notes**:
    /// - After stop returns, worker threads will have been requested to stop and any existing handles will be joined.
    /// - This method swallows join errors and does not return a failure result.
    pub fn stop(&mut self) {
        // signal threads to stop
        self._stop_signal.store(true, Ordering::SeqCst);

        // take the handles
        let _polling_handle = self._polling_handle.take();
        let _command_handle = self._command_handle.take();

        // spawn a short-lived thread to join them so main thread is not blocked
        let _ = thread::spawn(move || {
            if let Some(h) = _polling_handle {
                let _ = h.join();
            }
            if let Some(h) = _command_handle {
                let _ = h.join();
            }
        }).join();
    }
}