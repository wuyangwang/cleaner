use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::targets;

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub max_depth: usize,
    pub skip_patterns: Vec<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            skip_patterns: vec![
                "/registry/cache/".to_string(),
                "/registry/index/".to_string(),
                "/git/db/".to_string(),
                "/.cargo/bin/".to_string(),
                "/pkg/mod/cache/".to_string(),
                "/.m2/repository/".to_string(),
                "/.gradle/caches/modules-2/".to_string(),
                "/.pnpm-store/".to_string(),
                "/AppData/Local/pnpm-store/".to_string(),
                "/pip/cache/selfcheck.json".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub path: PathBuf,
    pub size: u64,
    #[allow(dead_code)]
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

pub fn shorten_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = targets::get_home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(&*home_str) {
            return path_str.replacen(&*home_str, "~", 1);
        }
    }
    path_str.to_string()
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

fn should_skip_path(path: &Path, skip_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    skip_patterns.iter().any(|p| path_str.contains(p))
}

pub fn scan_directory(dir: &Path, category: &str) -> Result<Vec<TrashItem>> {
    scan_directory_with_config(dir, category, &ScanConfig::default())
}

pub fn scan_directory_with_config(
    dir: &Path,
    category: &str,
    config: &ScanConfig,
) -> Result<Vec<TrashItem>> {
    let mut items = Vec::new();

    if is_system_critical(dir) {
        return Ok(items);
    }

    for entry in WalkDir::new(dir)
        .max_depth(config.max_depth)
        .into_iter()
        .filter_entry(|e| {
            // 允许根目录即使是隐藏的（如 .npm），但跳过子目录中的隐藏文件
            if e.path() == dir {
                return true;
            }
            !is_hidden(e)
        })
        .flatten()
    {
        let path = entry.path();

        if is_system_critical(path) || should_skip_path(path, &config.skip_patterns) {
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
        "/", "/bin", "/boot", "/dev", "/etc", "/lib", "/lib64", "/proc", "/root", "/sbin", "/sys",
        "/usr", "/var",
    ];

    let path_str = path.to_string_lossy().to_string();

    for critical in critical_paths {
        if path_str == critical || path_str.starts_with(&format!("{}/", critical)) {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let windows_critical = vec![
            "Windows",
            "Program Files",
            "Program Files (x86)",
            "ProgramData",
        ];

        if let Some(components) = path.components().next() {
            let root = components.as_os_str().to_string_lossy();
            if root.len() == 2 && root.ends_with(':') {
                let root_with_sep = format!("{}\\", root);
                if path_str == root_with_sep || path_str == root {
                    return true;
                }

                for critical in &windows_critical {
                    let critical_path = format!("{}\\{}", root, critical);
                    if path_str == critical_path
                        || path_str.starts_with(&format!("{}\\", critical_path))
                    {
                        return true;
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::{ScanConfig, is_system_critical};
    use std::path::Path;

    #[test]
    fn system_critical_root_is_blocked() {
        assert!(is_system_critical(Path::new("/")));
        assert!(is_system_critical(Path::new("/usr/bin")));
    }

    #[test]
    fn normal_home_subdir_is_not_system_critical() {
        assert!(!is_system_critical(Path::new("/tmp/cleaner-test-file")));
    }

    #[test]
    fn default_scan_config_has_expected_limits() {
        let config = ScanConfig::default();
        assert_eq!(config.max_depth, 5);
        assert!(config.skip_patterns.iter().any(|p| p == "/.m2/repository/"));
    }
}
