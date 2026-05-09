use anyhow::Result;
use std::path::Path;
use sysinfo::Disks;

pub struct DiskInfo {
    pub available: u64,
}

impl DiskInfo {
    pub fn available_str(&self) -> String {
        crate::scanner::format_size(self.available)
    }
}

pub fn get_disk_info(path: &Path) -> Result<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let path_str = path.to_string_lossy();

    for disk in disks.list() {
        let mount_point = disk.mount_point().to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            let drive_letter = path_str.chars().take(2).collect::<String>();
            if mount_point.starts_with(&drive_letter) {
                return Ok(DiskInfo {
                    available: disk.available_space(),
                });
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if path_str.starts_with(&*mount_point) {
                return Ok(DiskInfo {
                    available: disk.available_space(),
                });
            }
        }
    }

    // 回退到根目录
    for disk in disks.list() {
        if disk.mount_point().to_string_lossy() == "/" {
            return Ok(DiskInfo {
                available: disk.available_space(),
            });
        }
    }

    anyhow::bail!("无法获取磁盘信息")
}

pub fn calculate_freed(before: &DiskInfo, after: &DiskInfo) -> u64 {
    after.available.saturating_sub(before.available)
}
