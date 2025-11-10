#[cfg(test)]
mod clipboard_manager_test {
    use serial_test::serial;
    use core::panic;
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
        common::ClipboardErr, 
        services::clipboard_manager::Manager
    };

    #[test]
    #[serial]
    fn test_poller_stops_on_signal() {
        // Create new manager
        let mut manager = Manager::new().unwrap();

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

    // #[test]
    // #[serial]
    // fn test_manager_multi_spawn() {
    //     // Spawn a manager
    //     let _ = Manager::new();

    //     // Spawn another manager
    //     let err_manager = Manager::new();

    //     // Check if 
    //     match err_manager {
    //         Ok(_) => {
    //             panic!("MANAGER SHOULD NOT HAVE BEEN STARTED. MULTIPLE MANAGERS SPAWNED!")
    //         }
    //         Err(err) => {
    //             assert_eq!(err, ClipboardErr::ManagerMultiSpawn);
    //         }
    //     }
    // }

    // #[test]
    // #[serial]
    // fn test_manager_unlock() {
    //     // Spawn a manager
    //     let mut manager: Manager = Manager::new().unwrap();

    //     // close the manager
    //     manager.stop();

    //     // Spawn a second manager 
    //     match Manager::new() {
    //         Ok(_) => {/* Passed */},
    //         Err(_) => {panic!("MANAGER DID NOT SPAWN! PREVIOUS MANAGER NOT CLEANED!")},
    //     };

    // }

    // Add more tests here...

}