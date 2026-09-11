use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
}

pub fn herdr_config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("HERDR_CONFIG_PATH") {
        return PathBuf::from(custom);
    }
    let local = home_dir().join(".config/herdr/config.local.toml");
    if local.exists() {
        return local;
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
        PathBuf::from(format!("/run/user/{}/herdr/herdr.sock", unsafe {
            libc::getuid()
        })),
        PathBuf::from(format!("/tmp/herdr-{}.sock", unsafe { libc::getuid() })),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}

#[cfg(test)]
pub(crate) static TEST_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) struct EnvVarGuard {
    key: std::ffi::OsString,
    original: Option<std::ffi::OsString>,
}

#[cfg(test)]
impl EnvVarGuard {
    pub(crate) fn set(
        key: impl AsRef<std::ffi::OsStr>,
        value: impl AsRef<std::ffi::OsStr>,
    ) -> Self {
        let key_os = key.as_ref().to_os_string();
        let original = std::env::var_os(&key_os);
        unsafe {
            std::env::set_var(&key_os, value);
        }
        Self {
            key: key_os,
            original,
        }
    }

    pub(crate) fn remove(key: impl AsRef<std::ffi::OsStr>) -> Self {
        let key_os = key.as_ref().to_os_string();
        let original = std::env::var_os(&key_os);
        unsafe {
            std::env::remove_var(&key_os);
        }
        Self {
            key: key_os,
            original,
        }
    }
}

#[cfg(test)]
impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref val) = self.original {
                std::env::set_var(&self.key, val);
            } else {
                std::env::remove_var(&self.key);
            }
        }
    }
}

#[cfg(test)]
pub(crate) struct TempDir {
    pub(crate) path: PathBuf,
}

#[cfg(test)]
impl TempDir {
    pub(crate) fn new(name: &str) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "herdr_test_{}_{}_{}_{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            id
        ));
        std::fs::create_dir_all(&dir).expect("failed to create temporary test directory");
        Self { path: dir }
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[cfg(test)]
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
#[path = "paths_unit.rs"]
mod tests;
