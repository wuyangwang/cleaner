use crate::disk::{self, DiskInfo};
use crate::scanner::{self, TrashItem};
use anyhow::Result;
use std::path::PathBuf;

pub enum AppState {
    Scanning,
    Selecting,
    Confirming,
    Cleaning,
    Complete,
}

pub struct App {
    pub items: Vec<TrashItem>,
    pub state: AppState,
    pub selected_index: usize,
    pub disk_before: Option<DiskInfo>,
    pub disk_after: Option<DiskInfo>,
    pub cleaned_size: u64,
    pub clean_progress: usize,
    pub clean_total: usize,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: AppState::Scanning,
            selected_index: 0,
            disk_before: None,
            disk_after: None,
            cleaned_size: 0,
            clean_progress: 0,
            clean_total: 0,
            error_message: None,
        }
    }

    pub fn scan(&mut self) -> Result<()> {
        self.state = AppState::Scanning;

        // 获取磁盘信息（清理前）
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.disk_before = Some(disk::get_disk_info(&home)?);

        // 扫描垃圾文件
        self.items = scanner::scan_trash_dirs()?;

        if self.items.is_empty() {
            self.state = AppState::Complete;
        } else {
            self.state = AppState::Selecting;
        }

        Ok(())
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_index) {
            item.selected = !item.selected;
        }
    }

    pub fn select_all(&mut self) {
        for item in &mut self.items {
            item.selected = true;
        }
    }

    pub fn deselect_all(&mut self) {
        for item in &mut self.items {
            item.selected = false;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn get_selected_count(&self) -> usize {
        self.items.iter().filter(|i| i.selected).count()
    }

    pub fn get_selected_size(&self) -> u64 {
        scanner::get_total_size(&self.items)
    }

    pub fn start_clean(&mut self) {
        self.state = AppState::Cleaning;
        self.clean_progress = 0;
        self.clean_total = self.get_selected_count();
    }

    pub fn clean_next(&mut self) -> Result<bool> {
        let selected: Vec<usize> = self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.selected)
            .map(|(i, _)| i)
            .collect();

        if self.clean_progress >= selected.len() {
            return Ok(true);
        }

        let idx = selected[self.clean_progress];
        let item = &self.items[idx];

        // 安全验证：再次检查路径是否安全
        if crate::scanner::is_system_critical(&item.path) {
            self.error_message = Some(format!("拒绝删除系统路径: {}", item.path.display()));
            self.clean_progress += 1;
            return Ok(self.clean_progress >= selected.len());
        }

        if item.path.exists() {
            if item.path.is_dir() {
                std::fs::remove_dir_all(&item.path)?;
            } else {
                std::fs::remove_file(&item.path)?;
            }
        }

        self.cleaned_size += item.size;
        self.clean_progress += 1;

        Ok(self.clean_progress >= selected.len())
    }

    pub fn finish_clean(&mut self) -> Result<()> {
        // 获取清理后磁盘信息
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.disk_after = Some(disk::get_disk_info(&home)?);

        // 移除已删除的项
        self.items.retain(|item| !item.selected || !item.path.exists());

        self.state = AppState::Complete;
        Ok(())
    }

    pub fn get_disk_freed(&self) -> u64 {
        match (&self.disk_before, &self.disk_after) {
            (Some(before), Some(after)) => disk::calculate_freed(before, after),
            _ => 0,
        }
    }

    pub fn get_disk_freed_str(&self) -> String {
        scanner::format_size(self.get_disk_freed())
    }
}
