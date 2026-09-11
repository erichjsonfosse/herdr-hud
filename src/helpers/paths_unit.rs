use super::*;

#[test]
fn test_home_dir_returns_valid_path() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("home_valid");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    let home = home_dir();
    assert_eq!(home, temp.path());
    assert!(!home.as_os_str().is_empty());
}

#[test]
fn test_home_dir_fallback_when_unset() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _home_guard = EnvVarGuard::remove("HOME");

    let home = home_dir();
    assert_eq!(home, PathBuf::from("."));
}

#[test]
fn test_herdr_config_path_default() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("herdr_config_default");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());
    let _config_guard = EnvVarGuard::remove("HERDR_CONFIG_PATH");

    let expected = temp.path().join(".config/herdr/config.toml");
    assert_eq!(herdr_config_path(), expected);
}

#[test]
fn test_herdr_config_path_with_env() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("herdr_config_custom");
    let custom_path = temp.path().join("my_herdr_config.toml");
    let _config_guard = EnvVarGuard::set("HERDR_CONFIG_PATH", &custom_path);

    assert_eq!(herdr_config_path(), custom_path);
}

#[test]
fn test_plugin_config_path_default() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("plugin_config_default");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());
    let _plugin_guard = EnvVarGuard::remove("HERDR_STATUS_BAR_CONFIG");

    let expected = temp.path().join(".config/herdr/status-bar.json");
    assert_eq!(plugin_config_path(), expected);
}

#[test]
fn test_plugin_config_path_with_env() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("plugin_config_custom");
    let custom_path = temp.path().join("my_status_bar.json");
    let _plugin_guard = EnvVarGuard::set("HERDR_STATUS_BAR_CONFIG", &custom_path);

    assert_eq!(plugin_config_path(), custom_path);
}

#[test]
fn test_discover_socket_with_herdr_socket_env_mock_socket() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("mock_socket");
    let sock_path = temp.path().join("mock.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&sock_path)
        .expect("failed to bind mock unix domain socket");
    let _socket_guard = EnvVarGuard::set("HERDR_SOCKET", &sock_path);

    assert_eq!(discover_socket(), Some(sock_path));
}

#[test]
fn test_discover_socket_with_herdr_socket_env_file() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("file_socket");
    let sock_path = temp.path().join("file.sock");
    std::fs::write(&sock_path, b"test socket file").expect("failed to create file");
    let _socket_guard = EnvVarGuard::set("HERDR_SOCKET", &sock_path);

    assert_eq!(discover_socket(), Some(sock_path));
}

#[test]
fn test_discover_socket_env_non_existent_falls_back_to_candidates() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("socket_env_fallback");
    let non_existent = temp.path().join("non_existent.sock");
    let _socket_guard = EnvVarGuard::set("HERDR_SOCKET", &non_existent);

    let candidate_sock = temp.path().join(".herdr/herdr.sock");
    std::fs::create_dir_all(candidate_sock.parent().unwrap()).unwrap();
    std::fs::write(&candidate_sock, b"").unwrap();
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    assert_eq!(discover_socket(), Some(candidate_sock));
}

#[test]
fn test_discover_socket_fallback_candidate_herdr_dir() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("candidate_herdr_dir");
    let _socket_guard = EnvVarGuard::remove("HERDR_SOCKET");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    let candidate_sock = temp.path().join(".herdr/herdr.sock");
    std::fs::create_dir_all(candidate_sock.parent().unwrap()).unwrap();
    std::fs::write(&candidate_sock, b"").unwrap();

    assert_eq!(discover_socket(), Some(candidate_sock));
}

#[test]
fn test_discover_socket_fallback_candidate_config_herdr_dir() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("candidate_config_dir");
    let _socket_guard = EnvVarGuard::remove("HERDR_SOCKET");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    let candidate_sock = temp.path().join(".config/herdr/herdr.sock");
    std::fs::create_dir_all(candidate_sock.parent().unwrap()).unwrap();
    std::fs::write(&candidate_sock, b"").unwrap();

    assert_eq!(discover_socket(), Some(candidate_sock));
}

#[test]
fn test_discover_socket_fallback_candidate_order() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("candidate_order");
    let _socket_guard = EnvVarGuard::remove("HERDR_SOCKET");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    let first_sock = temp.path().join(".herdr/herdr.sock");
    let second_sock = temp.path().join(".config/herdr/herdr.sock");
    std::fs::create_dir_all(first_sock.parent().unwrap()).unwrap();
    std::fs::create_dir_all(second_sock.parent().unwrap()).unwrap();
    std::fs::write(&first_sock, b"").unwrap();
    std::fs::write(&second_sock, b"").unwrap();

    assert_eq!(discover_socket(), Some(first_sock));
}

#[test]
fn test_discover_socket_none_when_no_socket_exists() {
    let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp = TempDir::new("no_socket");
    let _socket_guard = EnvVarGuard::remove("HERDR_SOCKET");
    let _home_guard = EnvVarGuard::set("HOME", temp.path());

    let uid = unsafe { libc::getuid() };
    let run_sock = PathBuf::from(format!("/run/user/{}/herdr/herdr.sock", uid));
    let tmp_sock = PathBuf::from(format!("/tmp/herdr-{}.sock", uid));

    let result = discover_socket();
    if !run_sock.exists() && !tmp_sock.exists() {
        assert_eq!(result, None);
    } else {
        assert!(result == Some(run_sock) || result == Some(tmp_sock));
    }
}

#[test]
fn test_env_var_guard_preserves_non_utf8() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let _lock = TEST_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let var_name = "TEST_HERDR_NON_UTF8_KEY";
        let non_utf8_initial = std::ffi::OsStr::from_bytes(&[0x61, 0x62, 0x80, 0x63]);
        let non_utf8_target = std::ffi::OsStr::from_bytes(&[0x78, 0x79, 0xff, 0x7a]);

        // Pre-set initial non-UTF8 value
        unsafe {
            std::env::set_var(var_name, non_utf8_initial);
        }

        {
            let _guard = EnvVarGuard::set(var_name, non_utf8_target);
            assert_eq!(
                std::env::var_os(var_name),
                Some(non_utf8_target.to_os_string())
            );
        }

        // After guard drops, previous non-UTF8 value should be restored
        assert_eq!(
            std::env::var_os(var_name),
            Some(non_utf8_initial.to_os_string())
        );

        // Clean up
        unsafe {
            std::env::remove_var(var_name);
        }
    }
}
