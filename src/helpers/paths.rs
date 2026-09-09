use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
}

pub fn herdr_config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("HERDR_CONFIG_PATH") {
        return PathBuf::from(custom);
    }
    home_dir().join(".config/herdr/config.toml")
}

pub fn plugin_config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("HERDR_STATUS_BAR_CONFIG") {
        return PathBuf::from(custom);
    }
    home_dir().join(".config/herdr/status-bar.json")
}

pub fn discover_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("HERDR_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let home = home_dir();
    let candidates = [
        home.join(".herdr/herdr.sock"),
        home.join(".config/herdr/herdr.sock"),
        PathBuf::from(format!(
            "/run/user/{}/herdr/herdr.sock",
            unsafe { libc::getuid() }
        )),
        PathBuf::from(format!("/tmp/herdr-{}.sock", unsafe { libc::getuid() })),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
#[path = "paths_unit.rs"]
mod tests;
