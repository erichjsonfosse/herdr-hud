use super::*;

#[test]
fn test_terminal_guard_initialization_or_unsupported() {
    // When run in a non-interactive test environment without a TTY,
    // enable_raw_mode() may fail with ENOTTY (Inappropriate ioctl for device).
    // The guard must either succeed or return a clean io::Error without panicking.
    match TerminalGuard::new(false) {
        Ok(guard) => {
            assert!(!guard.mouse_capture());
            drop(guard);
        }
        Err(e) => {
            assert!(
                e.raw_os_error().is_some() || e.kind() != std::io::ErrorKind::Other,
                "Expected standard I/O error on non-TTY: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_terminal_guard_with_mouse_capture_initialization_or_unsupported() {
    match TerminalGuard::new(true) {
        Ok(guard) => {
            assert!(guard.mouse_capture());
            drop(guard);
        }
        Err(e) => {
            assert!(
                e.raw_os_error().is_some() || e.kind() != std::io::ErrorKind::Other,
                "Expected standard I/O error on non-TTY: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_restore_terminal_does_not_panic() {
    restore_terminal(false);
    restore_terminal(true);
}
