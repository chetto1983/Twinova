//! The file a client finds the open app by: its port and its launch's token, in the app's data folder, readable by its
//! user only (on Unix mode 0600; on Windows the user's own AppData).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlFile {
    pub port: u16,
    pub token: String,
    /// The app's process, so an app quitting removes its own file and not a newer one's.
    pub pid: u32,
}

/// `control/current.json` in the app's data folder (Tauri's `app_data_dir`: the system's data folder and the app's
/// identifier).
pub fn path(data_dir: &Path) -> PathBuf {
    data_dir.join("control").join("current.json")
}

impl ControlFile {
    /// Writes the file whole, through a temporary file renamed over it, so a client never reads half of it.
    pub fn write(&self, data_dir: &Path) -> io::Result<PathBuf> {
        let target = path(data_dir);
        let folder = target.parent().unwrap_or(data_dir);
        fs::create_dir_all(folder)?;
        let temporary = folder.join(format!("current.{}.tmp", self.pid));
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let json = serde_json::to_vec(self).map_err(io::Error::other)?;
        io::Write::write_all(&mut options.open(&temporary)?, &json)?;
        fs::rename(&temporary, &target)?;
        Ok(target)
    }

    pub fn read(data_dir: &Path) -> io::Result<Self> {
        let bytes = fs::read(path(data_dir))?;
        serde_json::from_slice(&bytes).map_err(io::Error::other)
    }

    /// Removes the file when it is still this app's.
    pub fn remove(&self, data_dir: &Path) {
        if Self::read(data_dir).is_ok_and(|current| current.pid == self.pid) {
            let _ = fs::remove_file(path(data_dir));
        }
    }
}
