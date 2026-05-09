use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
    pub category: String,
}

impl TrashItem {
    pub fn new(path: PathBuf, size: u64, category: String) -> Self {
        Self {
            path,
            size,
            selected: false,
            category,
        }
    }

    pub fn size_str(&self) -> String {
        format_size(self.size)
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

    // 获取平台特定的垃圾目录
    let trash_dirs = get_trash_directories();

    for (dir, category) in trash_dirs {
        if dir.exists() {
            items.extend(scan_directory(&dir, &category)?);
        }
    }

    items.sort_by_key(|b| std::cmp::Reverse(b.size));

    Ok(items)
}

fn get_trash_directories() -> Vec<(PathBuf, String)> {
    let mut dirs = Vec::new();

    // 跨平台通用临时目录
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        dirs.push((PathBuf::from("/tmp"), "临时文件".to_string()));
        dirs.push((PathBuf::from("/var/tmp"), "临时文件".to_string()));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(temp) = std::env::var("TEMP") {
            dirs.push((PathBuf::from(temp), "临时文件".to_string()));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            dirs.push((PathBuf::from(&local_app_data).join("Temp"), "临时文件".to_string()));
        }
    }

    // 获取用户主目录
    let home = get_home_dir();

    if let Some(home) = home {
        // JavaScript / Node.js
        dirs.push((home.join(".npm"), "npm 缓存".to_string()));
        dirs.push((home.join(".pnpm-store"), "pnpm 缓存".to_string()));
        dirs.push((home.join("AppData").join("Local").join("pnpm-store"), "pnpm 缓存".to_string()));
        dirs.push((home.join(".yarn"), "yarn 缓存".to_string()));
        dirs.push((home.join(".cache").join("yarn"), "yarn 缓存".to_string()));
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                dirs.push((PathBuf::from(&local_app_data).join("Yarn"), "yarn 缓存".to_string()));
            }
        }
        dirs.push((home.join(".bun"), "bun 缓存".to_string()));
        dirs.push((home.join(".node_modules"), "node_modules".to_string()));

        // Rust / Cargo
        let cargo_home = std::env::var("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cargo"));
        dirs.push((cargo_home.join("registry"), "cargo 注册表".to_string()));
        dirs.push((cargo_home.join("git"), "cargo git".to_string()));
        dirs.push((cargo_home.join("target"), "cargo 构建".to_string()));

        // Go
        let gopath = std::env::var("GOPATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join("go"));
        dirs.push((gopath.join("pkg").join("mod"), "go 模块".to_string()));
        dirs.push((gopath.join("pkg").join("sumdb"), "go sumdb".to_string()));
        let gomodcache = std::env::var("GOMODCACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| gopath.join("pkg").join("mod"));
        dirs.push((gomodcache, "go 模块缓存".to_string()));
        let go_cache = std::env::var("GOCACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cache").join("go-build"));
        dirs.push((go_cache, "go 构建缓存".to_string()));

        // Python
        dirs.push((home.join(".cache").join("pip"), "pip 缓存".to_string()));
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                dirs.push((PathBuf::from(&local_app_data).join("pip").join("cache"), "pip 缓存".to_string()));
            }
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches").join("pip"), "pip 缓存".to_string()));
        }
        dirs.push((home.join(".conda").join("pkgs"), "conda 缓存".to_string()));
        dirs.push((home.join("Anaconda3").join("pkgs"), "conda 缓存".to_string()));
        dirs.push((home.join("miniconda3").join("pkgs"), "conda 缓存".to_string()));
        dirs.push((home.join(".cache").join("pypoetry"), "poetry 缓存".to_string()));
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches").join("pypoetry"), "poetry 缓存".to_string()));
        }
        dirs.push((home.join(".cache").join("pdm"), "pdm 缓存".to_string()));
        dirs.push((home.join(".cache").join("virtualenv"), "虚拟环境".to_string()));

        // Java
        dirs.push((home.join(".m2").join("repository"), "maven 缓存".to_string()));
        dirs.push((home.join(".m2").join("wrapper"), "maven 包装器".to_string()));
        dirs.push((home.join(".gradle").join("caches"), "gradle 缓存".to_string()));
        dirs.push((home.join(".gradle").join("wrapper"), "gradle 包装器".to_string()));
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches").join("Gradle"), "gradle 缓存".to_string()));
        }
        dirs.push((home.join(".sbt"), "sbt 缓存".to_string()));
        dirs.push((home.join(".ivy2"), "ivy2 缓存".to_string()));
        dirs.push((home.join(".cache").join("coursier"), "coursier 缓存".to_string()));

        // .NET / C#
        dirs.push((home.join(".nuget").join("packages"), "nuget 缓存".to_string()));
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                dirs.push((PathBuf::from(&local_app_data).join("NuGet").join("Cache"), "nuget 缓存".to_string()));
            }
        }
        dirs.push((home.join(".dotnet").join("tools"), "dotnet 工具".to_string()));

        // Ruby
        dirs.push((home.join(".bundle").join("cache"), "bundler 缓存".to_string()));
        dirs.push((home.join(".gem"), "gem 缓存".to_string()));
        dirs.push((home.join(".cache").join("gem"), "gem 缓存".to_string()));
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches").join("gem"), "gem 缓存".to_string()));
        }

        // PHP
        dirs.push((home.join(".composer").join("cache"), "composer 缓存".to_string()));
        dirs.push((home.join(".cache").join("composer"), "composer 缓存".to_string()));

        // Dart / Flutter
        dirs.push((home.join(".pub-cache"), "pub 缓存".to_string()));
        dirs.push((home.join(".dart"), "dart 缓存".to_string()));
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches").join("pub"), "pub 缓存".to_string()));
        }

        // Haskell
        dirs.push((home.join(".cabal").join("packages"), "cabal 缓存".to_string()));
        dirs.push((home.join(".stack").join("programs"), "stack 缓存".to_string()));

        // Elixir / Erlang
        dirs.push((home.join(".mix"), "mix 缓存".to_string()));
        dirs.push((home.join(".hex"), "hex 缓存".to_string()));

        // Zig
        dirs.push((home.join(".cache").join("zig"), "zig 缓存".to_string()));

        // 通用缓存
        #[cfg(target_os = "linux")]
        {
            dirs.push((home.join(".cache"), "用户缓存".to_string()));
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join("Library").join("Caches"), "用户缓存".to_string()));
        }

        // 回收站
        #[cfg(target_os = "linux")]
        {
            dirs.push((home.join(".local").join("share").join("Trash").join("files"), "回收站".to_string()));
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join(".Trash"), "废纸篓".to_string()));
        }
    }

    dirs
}

fn get_home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").map(PathBuf::from).ok()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        std::env::var("HOME").map(PathBuf::from).ok()
    }
}

fn scan_directory(dir: &Path, category: &str) -> Result<Vec<TrashItem>> {
    let mut items = Vec::new();

    // 安全检查：拒绝扫描系统关键目录
    if is_system_critical(dir) {
        return Ok(items);
    }

    for entry in WalkDir::new(dir)
        .max_depth(5) // 限制递归深度
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .flatten()
    {
        let path = entry.path();
        
        // 跳过系统关键目录
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
        "/", "/bin", "/boot", "/dev", "/etc", "/lib", "/lib64",
        "/proc", "/root", "/sbin", "/sys", "/usr", "/var",
        "C:\\", "C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)",
        "C:\\Users", "C:\\ProgramData",
    ];
    
    let path_str = path.to_string_lossy().to_string();
    
    // 检查精确匹配或前缀匹配
    for critical in critical_paths {
        if path_str == critical || path_str.starts_with(&format!("{}/", critical)) {
            return true;
        }
    }
    
    // 检查 home 目录本身（不包括子目录）
    if let Some(home) = get_home_dir()
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

pub fn get_total_size(items: &[TrashItem]) -> u64 {
    items.iter().filter(|i| i.selected).map(|i| i.size).sum()
}

#[allow(dead_code)]
pub fn get_selected_items(items: &[TrashItem]) -> Vec<&TrashItem> {
    items.iter().filter(|i| i.selected).collect()
}
