use std::path::PathBuf;

pub fn get_trash_directories() -> Vec<(PathBuf, String)> {
    let mut dirs = Vec::new();

    fn push_unique(dirs: &mut Vec<(PathBuf, String)>, path: PathBuf, label: String) {
        if dirs.iter().any(|(existing, _)| existing == &path) {
            return;
        }
        dirs.push((path, label));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        push_unique(&mut dirs, PathBuf::from("/tmp"), "临时文件".to_string());
        push_unique(&mut dirs, PathBuf::from("/var/tmp"), "临时文件".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(temp) = std::env::var("TEMP") {
            push_unique(&mut dirs, PathBuf::from(temp), "临时文件".to_string());
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            push_unique(
                &mut dirs,
                PathBuf::from(&local_app_data).join("Temp"),
                "临时文件".to_string(),
            );
        }

        // 扫描其他盘的临时目录
        for drive in &["D", "E", "F", "G", "H"] {
            let temp_path = format!("{}:\\Temp", drive);
            let path = PathBuf::from(&temp_path);
            if path.exists() {
                push_unique(&mut dirs, path, "临时文件".to_string());
            }
        }
    }

    let home = get_home_dir();

    if let Some(home) = home {
        // --- 1. 浏览器与系统缓存 (通常很大且扫描快) ---
        #[cfg(target_os = "linux")]
        {
            push_unique(
                &mut dirs,
                home.join(".cache").join("google-chrome"),
                "Chrome 浏览器缓存".to_string(),
            );
            push_unique(
                &mut dirs,
                home.join(".cache").join("thumbnails"),
                "缩略图缓存".to_string(),
            );
        }
        #[cfg(target_os = "macos")]
        {
            push_unique(
                &mut dirs,
                home.join("Library")
                    .join("Caches")
                    .join("Google")
                    .join("Chrome"),
                "Chrome 浏览器缓存".to_string(),
            );
            push_unique(
                &mut dirs,
                home.join("Library").join("Logs"),
                "系统日志".to_string(),
            );
        }

        // --- 2. 开发工具临时日志与解压源码 (安全清理) ---
        push_unique(
            &mut dirs,
            home.join(".npm").join("_logs"),
            "npm 日志".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".cache").join("Cypress"),
            "Cypress 缓存".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".cache").join("electron"),
            "Electron 缓存".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".cache").join("ms-playwright"),
            "Playwright 浏览器".to_string(),
        );
        #[cfg(target_os = "macos")]
        {
            push_unique(
                &mut dirs,
                home.join("Library").join("Caches").join("ms-playwright"),
                "Playwright 浏览器".to_string(),
            );
            push_unique(
                &mut dirs,
                home.join("Library").join("Caches").join("Homebrew"),
                "Homebrew 缓存".to_string(),
            );
            push_unique(
                &mut dirs,
                home.join("Library").join("Caches").join("CocoaPods"),
                "CocoaPods 缓存".to_string(),
            );
        }

        // --- 3. 包管理工具下载缓存 (副作用：清理后需重下) ---
        // JavaScript / Node.js
        push_unique(
            &mut dirs,
            home.join(".npm"),
            "npm 缓存 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".yarn"),
            "yarn 缓存 (清理后需重新下载)".to_string(),
        );
        push_unique(&mut dirs, home.join(".deno"), "deno 缓存".to_string());

        // Rust / Cargo
        let cargo_home = std::env::var("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cargo"));
        push_unique(
            &mut dirs,
            cargo_home.join("registry").join("src"),
            "Cargo 已解压源码 (可安全清理)".to_string(),
        );
        push_unique(
            &mut dirs,
            cargo_home.join("registry").join("cache"),
            "Cargo 下载缓存 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            cargo_home.join("git").join("db"),
            "Cargo git 仓库 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            cargo_home.join("git").join("checkouts"),
            "Cargo git 检出 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            cargo_home.join("target"),
            "Cargo 全局构建缓存 (清理后需重新编译)".to_string(),
        );

        // Go
        let gopath = std::env::var("GOPATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join("go"));
        push_unique(
            &mut dirs,
            gopath.join("pkg").join("mod"),
            "Go 模块 (清理后需重新下载)".to_string(),
        );
        let go_cache = std::env::var("GOCACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cache").join("go-build"));
        push_unique(
            &mut dirs,
            go_cache,
            "Go 构建缓存 (清理后需重新编译)".to_string(),
        );

        // Python
        push_unique(
            &mut dirs,
            home.join(".cache").join("pip"),
            "pip 缓存 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".cache").join("pypoetry"),
            "Poetry 缓存 (清理后需重新下载)".to_string(),
        );

        // Java
        push_unique(
            &mut dirs,
            home.join(".m2").join("repository"),
            "Maven 仓库 (慎删：重新下载极慢)".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".gradle").join("caches"),
            "Gradle 缓存 (清理后需重新下载)".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".gradle").join("wrapper"),
            "Gradle 包装器".to_string(),
        );

        // --- 4. 通用与系统回收站 ---
        #[cfg(target_os = "linux")]
        {
            push_unique(&mut dirs, home.join(".cache"), "用户通用缓存".to_string());
            push_unique(
                &mut dirs,
                home.join(".local")
                    .join("share")
                    .join("Trash")
                    .join("files"),
                "回收站".to_string(),
            );
        }
        #[cfg(target_os = "macos")]
        {
            push_unique(
                &mut dirs,
                home.join("Library").join("Caches"),
                "用户通用缓存".to_string(),
            );
            push_unique(&mut dirs, home.join(".Trash"), "废纸篓".to_string());
        }

        // Haskell
        push_unique(
            &mut dirs,
            home.join(".cabal").join("packages"),
            "cabal 缓存".to_string(),
        );
        push_unique(
            &mut dirs,
            home.join(".stack").join("programs"),
            "stack 缓存".to_string(),
        );

        // Elixir / Erlang
        push_unique(&mut dirs, home.join(".mix"), "mix 缓存".to_string());
        push_unique(&mut dirs, home.join(".hex"), "hex 缓存".to_string());

        // Zig
        push_unique(
            &mut dirs,
            home.join(".cache").join("zig"),
            "zig 缓存".to_string(),
        );

        // 通用缓存
        #[cfg(target_os = "linux")]
        {
            push_unique(&mut dirs, home.join(".cache"), "用户缓存".to_string());
        }
        #[cfg(target_os = "macos")]
        {
            push_unique(
                &mut dirs,
                home.join("Library").join("Caches"),
                "用户缓存".to_string(),
            );
        }

        // 回收站
        #[cfg(target_os = "linux")]
        {
            push_unique(
                &mut dirs,
                home.join(".local")
                    .join("share")
                    .join("Trash")
                    .join("files"),
                "回收站".to_string(),
            );
        }
        #[cfg(target_os = "macos")]
        {
            push_unique(&mut dirs, home.join(".Trash"), "废纸篓".to_string());
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

#[cfg(test)]
mod tests {
    use super::get_trash_directories;
    use std::collections::HashSet;

    #[test]
    fn trash_directories_do_not_repeat_same_path() {
        let dirs = get_trash_directories();
        let unique: HashSet<_> = dirs.iter().map(|(path, _)| path.clone()).collect();
        assert_eq!(unique.len(), dirs.len());
    }
}
