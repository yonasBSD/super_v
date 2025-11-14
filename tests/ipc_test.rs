#[cfg(test)]
mod ipc_tests {
    use std::fs::remove_file;

    use serial_test::serial;
    use super_v::{
        common::{IPCServerError, SOCKET_PATH},
        services::clipboard_ipc_server::{create_bind, create_default_stream},
    };

    #[test]
    #[serial]
    fn test_create_bind_success() {
        // Create a new listener
        let listener = create_bind();
        assert!(listener.is_ok(), "Failed to create and bind listener");
    }

    #[test]
    #[serial]
    fn test_create_bind_already_running() {
        // Create first listener
        let _listener1 = create_bind().unwrap();

        // Try to create second listener - should fail
        let listener2 = create_bind();
        match listener2 {
            Ok(_) => {
                panic!("Server should not be created. Two instances running!");
            }
            Err(err) => {
                assert_eq!(
                    err,
                    IPCServerError::BindError("IPC Server appears to be already running".into())
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_stream_connect_no_file() {
        let _ = remove_file(SOCKET_PATH);

        let stream = create_default_stream();

        match stream {
            Ok(_) => {
                panic!(
                    "ELEMENT OF SURPRISE! Stream connection was successful despite having no file."
                );
            }
            Err(err) => {
                assert_eq!(err, IPCServerError::FileNotFound);
            }
        }
    }

    #[test]
    #[serial]
    fn test_stream_connect_server_not_running() {
        // Create listener out of scope
        {
            let _ = create_bind();
        }
        // Now the server should be stopped

        let stream = create_default_stream();

        match stream {
            Ok(_) => {
                panic!("Stream connection was successful despite server not running.");
            }
            Err(err) => {
                assert_eq!(
                    err,
                    IPCServerError::ConnectionError("Connection Refused by server.".into())
                );
            }
        }
    }

    // Sending and reading payload should already be tested via the Manager tests,
    // So no need for that here...
}
