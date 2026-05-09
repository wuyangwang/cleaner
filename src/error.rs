use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CleanError {
    /// 尝试删除系统关键目录
    SystemPathForbidden(PathBuf),
    /// 没有选择任何文件
    NoFilesSelected,
    /// 文件不存在
    FileNotFound(PathBuf),
    /// 没有权限删除
    PermissionDenied(PathBuf),
    /// 扫描失败
    ScanFailed(String),
    /// 磁盘信息获取失败
    DiskInfoFailed(String),
    /// 删除失败
    DeleteFailed(PathBuf, String),
}

impl fmt::Display for CleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanError::SystemPathForbidden(path) => {
                write!(f, "拒绝删除系统关键路径: {}", path.display())
            }
            CleanError::NoFilesSelected => {
                write!(f, "未选择任何文件，请先选择要清理的项目")
            }
            CleanError::FileNotFound(path) => {
                write!(f, "文件不存在: {}", path.display())
            }
            CleanError::PermissionDenied(path) => {
                write!(f, "没有权限删除: {}", path.display())
            }
            CleanError::ScanFailed(msg) => {
                write!(f, "扫描失败: {}", msg)
            }
            CleanError::DiskInfoFailed(msg) => {
                write!(f, "获取磁盘信息失败: {}", msg)
            }
            CleanError::DeleteFailed(path, reason) => {
                write!(f, "删除失败 {} - {}", path.display(), reason)
            }
        }
    }
}

impl std::error::Error for CleanError {}

impl From<anyhow::Error> for CleanError {
    fn from(err: anyhow::Error) -> Self {
        CleanError::ScanFailed(err.to_string())
    }
}

impl From<std::io::Error> for CleanError {
    fn from(err: std::io::Error) -> Self {
        CleanError::DeleteFailed(PathBuf::new(), err.to_string())
    }
}

#[allow(dead_code)]
pub type CleanResult<T> = Result<T, CleanError>;
