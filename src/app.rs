use crate::disk::{self, DiskInfo};
use crate::error::CleanError;
use crate::scanner::{self, TrashItem};
use crate::tree::{DisplayItem, TreeNode};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

pub enum AppState {
    Scanning,
    Selecting,
    Confirming,
    Cleaning,
    Complete,
}

pub enum ScanEvent {
    Scanning(String),
    Finished(Vec<TrashItem>),
    Error(String),
}

pub struct App {
    pub tree: Vec<TreeNode>,
    pub display_items: Vec<DisplayItem>,
    pub state: AppState,
    pub selected_index: usize,
    pub disk_before: Option<DiskInfo>,
    pub disk_after: Option<DiskInfo>,
    pub clean_progress: usize,
    pub clean_total: usize,
    pub error_message: Option<String>,
    pub scan_rx: Option<Receiver<ScanEvent>>,
    pub current_scanning: String,
    pub selected_paths_cache: Vec<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            display_items: Vec::new(),
            state: AppState::Scanning,
            selected_index: 0,
            disk_before: None,
            disk_after: None,
            clean_progress: 0,
            clean_total: 0,
            error_message: None,
            scan_rx: None,
            current_scanning: String::new(),
            selected_paths_cache: Vec::new(),
        }
    }

    pub fn refresh_display(&mut self) {
        self.display_items = crate::tree::flatten_tree(&self.tree, 0);
        if self.display_items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.display_items.len() {
            self.selected_index = self.display_items.len() - 1;
        }
    }

    pub fn scan(&mut self) -> Result<()> {
        self.state = AppState::Scanning;
        self.current_scanning = "准备扫描...".to_string();

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        self.disk_before = Some(disk::get_disk_info(&home)?);

        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        std::thread::spawn(move || {
            let trash_dirs = crate::targets::get_trash_directories();
            let mut all_items = Vec::new();

            for (dir, category) in trash_dirs {
                if dir.exists() {
                    let dir_display = scanner::shorten_path(&dir);
                    let _ = tx.send(ScanEvent::Scanning(format!(
                        "正在扫描: {} ({})",
                        dir_display, category
                    )));

                    match scanner::scan_directory(&dir, &category) {
                        Ok(items) => all_items.extend(items),
                        Err(e) => {
                            let clean_error = CleanError::ScanFailed(e.to_string());
                            let _ = tx.send(ScanEvent::Error(format!(
                                "扫描 {} 失败: {}",
                                dir.display(),
                                clean_error
                            )));
                        }
                    }
                }
            }

            let _ = tx.send(ScanEvent::Finished(all_items));
        });

        Ok(())
    }

    pub fn update_scan(&mut self) -> Result<bool> {
        let mut finished = false;
        let mut new_items = None;

        if let Some(rx) = &self.scan_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    ScanEvent::Scanning(path) => {
                        self.current_scanning = path;
                    }
                    ScanEvent::Finished(items) => {
                        new_items = Some(items);
                        finished = true;
                    }
                    ScanEvent::Error(e) => {
                        self.error_message = Some(e);
                    }
                }
            }
        }

        if let Some(items) = new_items {
            self.tree = crate::tree::build_tree(items);
            self.refresh_display();
        }

        if finished {
            self.scan_rx = None;
            if self.display_items.is_empty() {
                self.state = AppState::Complete;
            } else {
                self.state = AppState::Selecting;
                self.selected_index = 0;
            }
        }

        Ok(finished)
    }

    pub fn toggle_collapse(&mut self) {
        if let Some(item) = self.display_items.get(self.selected_index)
            && item.is_dir
        {
            let path = item.path.clone();
            Self::toggle_collapse_in_nodes(&mut self.tree, &path);
            self.refresh_display();
            if let Some(new_idx) = self.display_items.iter().position(|i| i.path == path) {
                self.selected_index = new_idx;
            } else if self.selected_index >= self.display_items.len() {
                self.selected_index = self.display_items.len().saturating_sub(1);
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
            let path = item.path.clone();
            let is_dir = item.is_dir;
            if is_dir {
                Self::toggle_dir_selection(&mut self.tree, &path);
            } else {
                Self::toggle_file_selection(&mut self.tree, &path);
            }
            crate::tree::recompute_aggregates(&mut self.tree);
            self.refresh_display();
            if let Some(new_idx) = self.display_items.iter().position(|i| i.path == path) {
                self.selected_index = new_idx;
            }
        }
    }

    fn toggle_dir_selection(nodes: &mut Vec<TreeNode>, path: &std::path::Path) -> bool {
        for node in nodes {
            if let TreeNode::Dir(dir) = node {
                if dir.path == path {
                    let all_selected = dir.selected_count == dir.file_count;
                    dir.select_all(!all_selected);
                    return true;
                }
                if Self::toggle_dir_selection(&mut dir.children, path) {
                    return true;
                }
            }
        }
        false
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
        let all_selected = self.get_selected_count() == self.get_total_count();
        let target_state = !all_selected;

        for node in &mut self.tree {
            match node {
                TreeNode::Dir(dir) => dir.select_all(target_state),
                TreeNode::File(file) => file.selected = target_state,
            }
        }
        crate::tree::recompute_aggregates(&mut self.tree);
        self.refresh_display();
    }

    pub fn get_total_count(&self) -> usize {
        self.tree
            .iter()
            .map(|n| match n {
                TreeNode::Dir(d) => d.file_count,
                TreeNode::File(_) => 1,
            })
            .sum()
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.display_items.is_empty() {
            self.selected_index = self.display_items.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.display_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.display_items.len();
        }
    }

    pub fn get_selected_count(&self) -> usize {
        self.tree
            .iter()
            .map(|n| match n {
                TreeNode::Dir(d) => d.selected_count,
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
                TreeNode::Dir(d) => d.selected_size,
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

    pub fn start_clean(&mut self) {
        self.state = AppState::Cleaning;
        self.clean_progress = 0;
        self.selected_paths_cache = self.collect_selected_paths();
        self.clean_total = self.selected_paths_cache.len();
    }

    pub fn clean_next(&mut self) -> Result<bool> {
        if self.clean_progress >= self.selected_paths_cache.len() {
            return Ok(true);
        }

        // 批量处理：每次处理 50 个文件以提高效率，同时保持 UI 刷新
        let batch_size = 50;
        let end = std::cmp::min(
            self.clean_progress + batch_size,
            self.selected_paths_cache.len(),
        );

        for i in self.clean_progress..end {
            let path = &self.selected_paths_cache[i];
            if crate::scanner::is_system_critical(path) {
                self.error_message =
                    Some(CleanError::SystemPathForbidden(path.clone()).to_string());
                continue;
            }

            if path.exists() {
                let result = if path.is_dir() {
                    std::fs::remove_dir_all(path)
                } else {
                    std::fs::remove_file(path)
                };

                if let Err(e) = result {
                    self.error_message =
                        Some(CleanError::DeleteFailed(path.clone(), e.to_string()).to_string());
                }
            }
        }

        self.clean_progress = end;

        // 如果全部清理完成，尝试一次性清理空目录
        if self.clean_progress >= self.selected_paths_cache.len() {
            self.cleanup_empty_parents(&self.selected_paths_cache);
        }

        Ok(self.clean_progress >= self.selected_paths_cache.len())
    }

    fn cleanup_empty_parents(&self, deleted_paths: &[PathBuf]) {
        use std::collections::HashSet;
        let mut parents_to_check = HashSet::new();
        for path in deleted_paths {
            if let Some(parent) = path.parent() {
                parents_to_check.insert(parent.to_path_buf());
            }
        }

        // 递归尝试清理空目录，直到没有目录被删除
        let mut changed = true;
        while changed {
            changed = false;
            let mut next_parents = HashSet::new();
            for parent in parents_to_check {
                if parent.exists()
                    && parent.is_dir()
                    && !crate::scanner::is_system_critical(&parent)
                    && std::fs::read_dir(&parent)
                        .map(|mut d| d.next().is_none())
                        .unwrap_or(false)
                    && std::fs::remove_dir(&parent).is_ok()
                {
                    changed = true;
                    if let Some(p) = parent.parent() {
                        next_parents.insert(p.to_path_buf());
                    }
                }
            }
            parents_to_check = next_parents;
        }
    }

    fn collect_selected_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        Self::collect_selected_from_nodes(&self.tree, &mut paths);
        paths
    }

    fn collect_selected_from_nodes(nodes: &[TreeNode], paths: &mut Vec<PathBuf>) {
        for node in nodes {
            match node {
                TreeNode::Dir(dir) => {
                    // 不再直接使用 remove_dir_all，而是递归收集所有文件
                    // 这样可以避免误删被 should_skip_path 跳过的文件
                    Self::collect_selected_from_nodes(&dir.children, paths);
                }
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
        if let Ok(now) = disk::get_disk_info(&home) {
            self.disk_before = Some(now);
        }
        self.disk_after = None;

        Self::remove_deleted_files(&mut self.tree);
        crate::tree::recompute_aggregates(&mut self.tree);
        self.refresh_display();

        self.state = AppState::Selecting;
        Ok(())
    }

    fn remove_deleted_files(nodes: &mut Vec<TreeNode>) {
        // 先递归处理子节点
        for node in nodes.iter_mut() {
            if let TreeNode::Dir(dir) = node {
                Self::remove_deleted_files(&mut dir.children);
            }
        }

        // 然后过滤掉已不存在的文件和目录，以及变为空的目录
        nodes.retain(|node| match node {
            TreeNode::File(f) => !f.selected || f.path.exists(),
            TreeNode::Dir(d) => {
                // 如果目录本身已不存在，或者目录在树中已无子节点，则移除
                d.path.exists() && !d.children.is_empty()
            }
        });
    }
}
