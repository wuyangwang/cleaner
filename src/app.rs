use crate::disk::{self, DiskInfo};
use crate::scanner;
use crate::tree::{DisplayItem, TreeNode};
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
    pub tree: Vec<TreeNode>,
    pub display_items: Vec<DisplayItem>,
    pub state: AppState,
    pub selected_index: usize,
    pub sort_descending: bool,
    pub disk_before: Option<DiskInfo>,
    pub disk_after: Option<DiskInfo>,
    pub clean_progress: usize,
    pub clean_total: usize,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            display_items: Vec::new(),
            state: AppState::Scanning,
            selected_index: 0,
            sort_descending: true,
            disk_before: None,
            disk_after: None,
            clean_progress: 0,
            clean_total: 0,
            error_message: None,
        }
    }

    pub fn refresh_display(&mut self) {
        self.display_items = crate::tree::flatten_tree(&self.tree, 0);
    }

    pub fn scan(&mut self) -> Result<()> {
        self.state = AppState::Scanning;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.disk_before = Some(disk::get_disk_info(&home)?);

        let items = scanner::scan_trash_dirs()?;
        self.tree = crate::tree::build_tree(items);
        self.refresh_display();

        if self.display_items.is_empty() {
            self.state = AppState::Complete;
        } else {
            self.state = AppState::Selecting;
            self.selected_index = 0;
        }

        Ok(())
    }

    pub fn toggle_collapse(&mut self) {
        if let Some(item) = self.display_items.get(self.selected_index) {
            if item.is_dir {
                let path = item.path.clone();
                Self::toggle_collapse_in_nodes(&mut self.tree, &path);
                self.refresh_display();
                if self.selected_index >= self.display_items.len() {
                    self.selected_index = self.display_items.len().saturating_sub(1);
                }
            }
        }
    }

    fn toggle_collapse_in_nodes(nodes: &mut Vec<TreeNode>, path: &std::path::Path) -> bool {
        for node in nodes {
            match node {
                TreeNode::Dir(dir) => {
                    if dir.path == path {
                        dir.collapsed = !dir.collapsed;
                        return true;
                    }
                    if Self::toggle_collapse_in_nodes(&mut dir.children, path) {
                        return true;
                    }
                }
                TreeNode::File(_) => {}
            }
        }
        false
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.display_items.get(self.selected_index) {
            if item.is_dir {
                self.toggle_collapse();
            } else {
                let path = item.path.clone();
                Self::toggle_file_selection(&mut self.tree, &path);
                self.refresh_display();
            }
        }
    }

    fn toggle_file_selection(nodes: &mut Vec<TreeNode>, path: &std::path::Path) -> bool {
        for node in nodes {
            match node {
                TreeNode::Dir(dir) => {
                    if Self::toggle_file_selection(&mut dir.children, path) {
                        return true;
                    }
                }
                TreeNode::File(file) => {
                    if file.path == path {
                        file.selected = !file.selected;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn select_all(&mut self) {
        for node in &mut self.tree {
            if let TreeNode::Dir(dir) = node {
                dir.select_all(true);
            }
        }
        self.refresh_display();
    }

    pub fn deselect_all(&mut self) {
        for node in &mut self.tree {
            if let TreeNode::Dir(dir) = node {
                dir.select_all(false);
            }
        }
        self.refresh_display();
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.display_items.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn get_selected_count(&self) -> usize {
        self.tree
            .iter()
            .map(|n| match n {
                TreeNode::Dir(d) => d.selected_count(),
                TreeNode::File(f) => {
                    if f.selected {
                        1
                    } else {
                        0
                    }
                }
            })
            .sum()
    }

    pub fn get_selected_size(&self) -> u64 {
        self.tree
            .iter()
            .map(|n| match n {
                TreeNode::Dir(d) => d.selected_size(),
                TreeNode::File(f) => {
                    if f.selected {
                        f.size
                    } else {
                        0
                    }
                }
            })
            .sum()
    }

    pub fn sort_label(&self) -> &str {
        if self.sort_descending {
            "最大优先"
        } else {
            "最小优先"
        }
    }

    pub fn start_clean(&mut self) {
        self.state = AppState::Cleaning;
        self.clean_progress = 0;
        self.clean_total = self.get_selected_count();
    }

    pub fn clean_next(&mut self) -> Result<bool> {
        let selected_paths: Vec<PathBuf> = self.collect_selected_paths();
        if self.clean_progress >= selected_paths.len() {
            return Ok(true);
        }

        let path = &selected_paths[self.clean_progress];
        if crate::scanner::is_system_critical(path) {
            self.error_message = Some(format!("拒绝删除系统路径: {}", path.display()));
            self.clean_progress += 1;
            return Ok(self.clean_progress >= selected_paths.len());
        }

        if path.exists() {
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }

        self.clean_progress += 1;
        Ok(self.clean_progress >= selected_paths.len())
    }

    fn collect_selected_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        Self::collect_selected_from_nodes(&self.tree, &mut paths);
        paths
    }

    fn collect_selected_from_nodes(nodes: &[TreeNode], paths: &mut Vec<PathBuf>) {
        for node in nodes {
            match node {
                TreeNode::Dir(dir) => Self::collect_selected_from_nodes(&dir.children, paths),
                TreeNode::File(file) => {
                    if file.selected {
                        paths.push(file.path.clone());
                    }
                }
            }
        }
    }

    pub fn finish_clean(&mut self) -> Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.disk_after = Some(disk::get_disk_info(&home)?);

        Self::remove_deleted_files(&mut self.tree);
        self.refresh_display();

        self.state = AppState::Complete;
        Ok(())
    }

    fn remove_deleted_files(nodes: &mut Vec<TreeNode>) {
        nodes.retain(|node| match node {
            TreeNode::File(f) => !f.selected || f.path.exists(),
            TreeNode::Dir(_) => true,
        });
        for node in nodes {
            if let TreeNode::Dir(dir) = node {
                Self::remove_deleted_files(&mut dir.children);
            }
        }
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
