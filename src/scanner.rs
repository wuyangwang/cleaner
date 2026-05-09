use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::targets;

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub path: PathBuf,
    pub size: u64,
    pub category: String,
}

impl TrashItem {
    pub fn new(path: PathBuf, size: u64, category: String) -> Self {
        Self {
            path,
            size,
            category,
        }
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn scan_trash_dirs() -> Result<Vec<TrashItem>> {
    let mut items = Vec::new();
    let trash_dirs = targets::get_trash_directories();

    for (dir, category) in trash_dirs {
        if dir.exists() {
            items.extend(scan_directory(&dir, &category)?);
        }
    }

    items.sort_by_key(|b| std::cmp::Reverse(b.size));
    Ok(items)
}

fn scan_directory(dir: &Path, category: &str) -> Result<Vec<TrashItem>> {
    let mut items = Vec::new();

    if is_system_critical(dir) {
        return Ok(items);
    }

    for entry in WalkDir::new(dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .flatten()
    {
        let path = entry.path();

        if is_system_critical(path) {
            continue;
        }

        if entry.file_type().is_file()
            && let Ok(metadata) = entry.metadata()
            && metadata.len() > 0
        {
            items.push(TrashItem::new(
                path.to_path_buf(),
                metadata.len(),
                category.to_string(),
            ));
        }
    }

    Ok(items)
}

pub fn is_system_critical(path: &Path) -> bool {
    let critical_paths: Vec<&str> = vec![
        "/",
        "/bin",
        "/boot",
        "/dev",
        "/etc",
        "/lib",
        "/lib64",
        "/proc",
        "/root",
        "/sbin",
        "/sys",
        "/usr",
        "/var",
        "C:\\",
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Program Files (x86)",
        "C:\\Users",
        "C:\\ProgramData",
    ];

    let path_str = path.to_string_lossy().to_string();

    for critical in critical_paths {
        if path_str == critical || path_str.starts_with(&format!("{}/", critical)) {
            return true;
        }
    }

    if let Some(home) = targets::get_home_dir()
        && path == home
    {
        return true;
    }

    false
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
