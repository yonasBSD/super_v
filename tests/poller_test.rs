#[cfg(test)]
mod clipboard_poller_test {
    use std::{
        thread,
        sync::{
            atomic::{
                Ordering
            }
        }, 
        time::Duration
    };
    use super_v::{
        services::clipboard_manager::Manager
    };

    #[test]
    fn test_poller_stops_on_signal() {
        // Create new manager
        let mut manager = Manager::new();

        // start polling
        manager._polling_service(); 

        // Give time
        thread::sleep(Duration::from_millis(50));

        // Send stop signal
        manager._stop_signal.store(true, Ordering::SeqCst);

        // Give time for the poller to check the signal and exit
        thread::sleep(Duration::from_millis(600));

        // Take the handle and match
        match manager._polling_handle.take() {
            Some(p_handle) => {
                // Check if it's still running. It should not be.
                assert!(p_handle.is_finished(), "Uh-oh! Poller still running after sending the Stop Signal");

                // Join the thread.
                let _ = p_handle.join();
            },
            None => {
                panic!("POLLING HANDLE EMPTY WHEN IT SHOULD NOT HAVE BEEN!");
            }
        }

    }

    // Add more tests here...

}