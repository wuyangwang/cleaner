use std::path::PathBuf;

pub fn get_trash_directories() -> Vec<(PathBuf, String)> {
    let mut dirs = Vec::new();

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
            dirs.push((
                PathBuf::from(&local_app_data).join("Temp"),
                "临时文件".to_string(),
            ));
        }

        // 扫描其他盘的临时目录
        for drive in &["D", "E", "F", "G", "H"] {
            let temp_path = format!("{}:\\Temp", drive);
            let path = PathBuf::from(&temp_path);
            if path.exists() {
                dirs.push((path, "临时文件".to_string()));
            }
        }
    }

    let home = get_home_dir();

    if let Some(home) = home {
        // --- 1. 浏览器与系统缓存 (通常很大且扫描快) ---
        #[cfg(target_os = "linux")]
        {
            dirs.push((
                home.join(".cache").join("google-chrome"),
                "Chrome 浏览器缓存".to_string(),
            ));
            dirs.push((
                home.join(".cache").join("thumbnails"),
                "缩略图缓存".to_string(),
            ));
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((
                home.join("Library")
                    .join("Caches")
                    .join("Google")
                    .join("Chrome"),
                "Chrome 浏览器缓存".to_string(),
            ));
            dirs.push((home.join("Library").join("Logs"), "系统日志".to_string()));
        }

        // --- 2. 开发工具临时日志与解压源码 (安全清理) ---
        dirs.push((home.join(".npm").join("_logs"), "npm 日志".to_string()));
        dirs.push((
            home.join(".cargo").join("registry").join("src"),
            "Cargo 已解压源码 (可安全清理)".to_string(),
        ));
        dirs.push((
            home.join(".cache").join("Cypress"),
            "Cypress 缓存".to_string(),
        ));
        dirs.push((
            home.join(".cache").join("electron"),
            "Electron 缓存".to_string(),
        ));
        dirs.push((
            home.join(".cache").join("ms-playwright"),
            "Playwright 浏览器".to_string(),
        ));
        #[cfg(target_os = "macos")]
        {
            dirs.push((
                home.join("Library").join("Caches").join("ms-playwright"),
                "Playwright 浏览器".to_string(),
            ));
            dirs.push((
                home.join("Library").join("Caches").join("Homebrew"),
                "Homebrew 缓存".to_string(),
            ));
            dirs.push((
                home.join("Library").join("Caches").join("CocoaPods"),
                "CocoaPods 缓存".to_string(),
            ));
        }

        // --- 3. 包管理工具下载缓存 (副作用：清理后需重下) ---
        // JavaScript / Node.js
        dirs.push((home.join(".npm"), "npm 缓存 (清理后需重新下载)".to_string()));
        dirs.push((
            home.join(".yarn"),
            "yarn 缓存 (清理后需重新下载)".to_string(),
        ));
        dirs.push((home.join(".deno"), "deno 缓存".to_string()));

        // Rust / Cargo
        let cargo_home = std::env::var("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cargo"));
        dirs.push((
            cargo_home.join("target"),
            "Cargo 全局构建缓存 (清理后需重新编译)".to_string(),
        ));

        // Go
        let gopath = std::env::var("GOPATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join("go"));
        dirs.push((
            gopath.join("pkg").join("mod"),
            "Go 模块 (清理后需重新下载)".to_string(),
        ));
        let go_cache = std::env::var("GOCACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cache").join("go-build"));
        dirs.push((go_cache, "Go 构建缓存 (清理后需重新编译)".to_string()));

        // Python
        dirs.push((
            home.join(".cache").join("pip"),
            "pip 缓存 (清理后需重新下载)".to_string(),
        ));
        dirs.push((
            home.join(".cache").join("pypoetry"),
            "Poetry 缓存 (清理后需重新下载)".to_string(),
        ));

        // Java
        dirs.push((
            home.join(".m2").join("repository"),
            "Maven 仓库 (慎删：重新下载极慢)".to_string(),
        ));
        dirs.push((
            home.join(".gradle").join("caches"),
            "Gradle 缓存 (清理后需重新下载)".to_string(),
        ));
        dirs.push((
            home.join(".gradle").join("wrapper"),
            "Gradle 包装器".to_string(),
        ));

        // --- 4. 通用与系统回收站 ---
        #[cfg(target_os = "linux")]
        {
            dirs.push((home.join(".cache"), "用户通用缓存".to_string()));
            dirs.push((
                home.join(".local")
                    .join("share")
                    .join("Trash")
                    .join("files"),
                "回收站".to_string(),
            ));
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((
                home.join("Library").join("Caches"),
                "用户通用缓存".to_string(),
            ));
            dirs.push((home.join(".Trash"), "废纸篓".to_string()));
        }

        // Haskell
        dirs.push((
            home.join(".cabal").join("packages"),
            "cabal 缓存".to_string(),
        ));
        dirs.push((
            home.join(".stack").join("programs"),
            "stack 缓存".to_string(),
        ));

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
            dirs.push((
                home.join(".local")
                    .join("share")
                    .join("Trash")
                    .join("files"),
                "回收站".to_string(),
            ));
        }
        #[cfg(target_os = "macos")]
        {
            dirs.push((home.join(".Trash"), "废纸篓".to_string()));
        }
    }

    dirs
}

pub fn get_home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").map(PathBuf::from).ok()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        std::env::var("HOME").map(PathBuf::from).ok()
    }
}
