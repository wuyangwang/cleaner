use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::scanner::TrashItem;

#[derive(Debug, Clone)]
pub enum TreeNode {
    Dir(DirNode),
    File(FileNode),
}

#[derive(Debug, Clone)]
pub struct DirNode {
    pub path: PathBuf,
    pub children: Vec<TreeNode>,
    pub collapsed: bool,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct DisplayItem {
    pub path: PathBuf,
    pub size: u64,
    pub depth: usize,
    pub is_dir: bool,
    pub collapsed: bool,
    pub file_count: usize,
    pub selected: bool,
}

impl DirNode {
    pub fn total_size(&self) -> u64 {
        self.children
            .iter()
            .map(|c| match c {
                TreeNode::Dir(d) => d.total_size(),
                TreeNode::File(f) => f.size,
            })
            .sum()
    }

    pub fn file_count(&self) -> usize {
        self.children
            .iter()
            .map(|c| match c {
                TreeNode::Dir(d) => d.file_count(),
                TreeNode::File(_) => 1,
            })
            .sum()
    }

    pub fn selected_size(&self) -> u64 {
        self.children
            .iter()
            .map(|c| match c {
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

    pub fn selected_count(&self) -> usize {
        self.children
            .iter()
            .map(|c| match c {
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

    pub fn select_all(&mut self, selected: bool) {
        for child in &mut self.children {
            match child {
                TreeNode::Dir(d) => d.select_all(selected),
                TreeNode::File(f) => f.selected = selected,
            }
        }
    }
}

pub fn build_tree(items: Vec<TrashItem>) -> Vec<TreeNode> {
    let mut groups: HashMap<PathBuf, Vec<TrashItem>> = HashMap::new();
    for item in items {
        let parent = item.path.parent().unwrap_or(Path::new("/")).to_path_buf();
        groups.entry(parent).or_default().push(item);
    }
    build_tree_from_groups(groups)
}

fn build_tree_from_groups(mut groups: HashMap<PathBuf, Vec<TrashItem>>) -> Vec<TreeNode> {
    let mut roots: Vec<TreeNode> = Vec::new();
    if groups.is_empty() {
        return roots;
    }

    let mut dir_map: HashMap<PathBuf, Vec<TreeNode>> = HashMap::new();

    for (parent, items) in groups.drain() {
        let children: Vec<TreeNode> = items
            .into_iter()
            .map(|item| {
                TreeNode::File(FileNode {
                    path: item.path,
                    size: item.size,
                    selected: false,
                })
            })
            .collect();
        dir_map.entry(parent).or_default().extend(children);
    }

    // Process deepest directories first, building bottom-up
    let mut sorted_dirs: Vec<PathBuf> = dir_map.keys().cloned().collect();
    sorted_dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    for dir_path in &sorted_dirs {
        let children = dir_map.remove(dir_path).unwrap_or_default();
        let parent = dir_path.parent();

        if let Some(parent_path) = parent {
            if parent_path != dir_path {
                let dir_node = TreeNode::Dir(DirNode {
                    path: dir_path.clone(),
                    children,
                    collapsed: true,
                });
                dir_map
                    .entry(parent_path.to_path_buf())
                    .or_default()
                    .push(dir_node);
            } else {
                roots.extend(children);
            }
        } else {
            roots.extend(children);
        }
    }

    sort_tree_nodes(&mut roots);
    roots
}

fn sort_tree_nodes(nodes: &mut Vec<TreeNode>) {
    nodes.sort_by(|a, b| match (a, b) {
        (TreeNode::Dir(_), TreeNode::File(_)) => std::cmp::Ordering::Less,
        (TreeNode::File(_), TreeNode::Dir(_)) => std::cmp::Ordering::Greater,
        (TreeNode::Dir(a), TreeNode::Dir(b)) => b.total_size().cmp(&a.total_size()),
        (TreeNode::File(a), TreeNode::File(b)) => b.size.cmp(&a.size),
    });

    for node in nodes {
        if let TreeNode::Dir(dir) = node {
            sort_tree_nodes(&mut dir.children);
        }
    }
}

pub fn flatten_tree(nodes: &[TreeNode], depth: usize) -> Vec<DisplayItem> {
    let mut items = Vec::new();
    for node in nodes {
        match node {
            TreeNode::Dir(dir) => {
                items.push(DisplayItem {
                    path: dir.path.clone(),
                    size: dir.total_size(),
                    depth,
                    is_dir: true,
                    collapsed: dir.collapsed,
                    file_count: dir.file_count(),
                    selected: false,
                });
                if !dir.collapsed {
                    items.extend(flatten_tree(&dir.children, depth + 1));
                }
            }
            TreeNode::File(file) => {
                items.push(DisplayItem {
                    path: file.path.clone(),
                    size: file.size,
                    depth,
                    is_dir: false,
                    collapsed: false,
                    file_count: 0,
                    selected: file.selected,
                });
            }
        }
    }
    items
}
