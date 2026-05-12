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
    pub total_size: u64,
    pub file_count: usize,
    pub selected_size: u64,
    pub selected_count: usize,
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
    pub selected_count: usize,
}

impl DirNode {
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
    let mut roots = build_tree_from_groups(groups);
    recompute_aggregates(&mut roots);
    roots
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

    let mut queue: Vec<PathBuf> = dir_map.keys().cloned().collect();
    queue.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    while let Some(dir_path) = queue.pop() {
        let Some(children) = dir_map.remove(&dir_path) else {
            continue;
        };
        let parent_path = dir_path.parent().map(|p| p.to_path_buf());

        if let Some(parent_path) = parent_path {
            if parent_path != dir_path {
                let dir_node = TreeNode::Dir(DirNode {
                    path: dir_path,
                    children,
                    collapsed: true,
                    total_size: 0,
                    file_count: 0,
                    selected_size: 0,
                    selected_count: 0,
                });
                let is_new = !dir_map.contains_key(&parent_path);
                dir_map
                    .entry(parent_path.clone())
                    .or_default()
                    .push(dir_node);
                if is_new {
                    queue.push(parent_path);
                }
            } else {
                roots.extend(children);
            }
        } else {
            roots.extend(children);
        }
    }

    sort_tree_nodes(&mut roots);
    merge_single_child_dirs(&mut roots);
    roots
}

fn sort_tree_nodes(nodes: &mut Vec<TreeNode>) {
    nodes.sort_by(|a, b| match (a, b) {
        (TreeNode::Dir(_), TreeNode::File(_)) => std::cmp::Ordering::Less,
        (TreeNode::File(_), TreeNode::Dir(_)) => std::cmp::Ordering::Greater,
        (TreeNode::Dir(a), TreeNode::Dir(b)) => b.total_size.cmp(&a.total_size),
        (TreeNode::File(a), TreeNode::File(b)) => b.size.cmp(&a.size),
    });

    for node in nodes {
        if let TreeNode::Dir(dir) = node {
            sort_tree_nodes(&mut dir.children);
        }
    }
}

fn merge_single_child_dirs(nodes: &mut Vec<TreeNode>) {
    for node in nodes {
        if let TreeNode::Dir(dir) = node {
            while dir.children.len() == 1 {
                if let TreeNode::Dir(child) = dir.children.remove(0) {
                    dir.path = dir.path.join(child.path.file_name().unwrap_or_default());
                    dir.children = child.children;
                } else {
                    break;
                }
            }
            merge_single_child_dirs(&mut dir.children);
        }
    }
}

pub fn flatten_tree(nodes: &[TreeNode], depth: usize) -> Vec<DisplayItem> {
    let mut items = Vec::new();
    for node in nodes {
        match node {
            TreeNode::Dir(dir) => {
                let total_size = dir.total_size;
                if total_size == 0 {
                    continue;
                }

                let selected_count = dir.selected_count;
                let file_count = dir.file_count;

                // 确定目录的选择状态：0-未选, 1-半选, 2-全选
                let selected_status = if selected_count == 0 {
                    0
                } else if selected_count == file_count {
                    2
                } else {
                    1
                };

                items.push(DisplayItem {
                    path: dir.path.clone(),
                    size: dir.total_size,
                    depth,
                    is_dir: true,
                    collapsed: dir.collapsed,
                    file_count,
                    selected: selected_status > 0, // 兼容旧字段
                    selected_count,
                });

                // 我们在 DisplayItem 里加一个新字段或者借用 selected 逻辑
                // 为了简单起见，我们稍后在 UI 层通过 selected_count 判断

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
                    selected_count: usize::from(file.selected),
                });
            }
        }
    }
    items
}

pub fn recompute_aggregates(nodes: &mut [TreeNode]) -> (u64, usize, u64, usize) {
    let mut total_size = 0;
    let mut file_count = 0;
    let mut selected_size = 0;
    let mut selected_count = 0;

    for node in nodes {
        match node {
            TreeNode::Dir(dir) => {
                let (t_size, t_count, s_size, s_count) = recompute_aggregates(&mut dir.children);
                dir.total_size = t_size;
                dir.file_count = t_count;
                dir.selected_size = s_size;
                dir.selected_count = s_count;
                total_size += t_size;
                file_count += t_count;
                selected_size += s_size;
                selected_count += s_count;
            }
            TreeNode::File(file) => {
                total_size += file.size;
                file_count += 1;
                if file.selected {
                    selected_size += file.size;
                    selected_count += 1;
                }
            }
        }
    }

    (total_size, file_count, selected_size, selected_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, size: u64) -> TrashItem {
        TrashItem::new(PathBuf::from(path), size, "test".to_string())
    }

    #[test]
    fn build_tree_groups_files_by_parent() {
        let nodes = build_tree(vec![item("/tmp/one/a.txt", 10), item("/tmp/two/b.txt", 20)]);
        assert_eq!(nodes.len(), 2);
        assert!(matches!(nodes[0], TreeNode::Dir(_)));
        assert!(matches!(nodes[1], TreeNode::Dir(_)));
    }

    #[test]
    fn flatten_tree_reflects_selection_state() {
        let mut nodes = build_tree(vec![item("/tmp/one/a.txt", 10), item("/tmp/one/b.txt", 20)]);
        if let TreeNode::Dir(dir) = &mut nodes[0] {
            if let TreeNode::File(file) = &mut dir.children[0] {
                file.selected = true;
            }
        }
        recompute_aggregates(&mut nodes);

        let items = flatten_tree(&nodes, 0);
        assert_eq!(items.len(), 1);
        assert!(items[0].selected);
        assert!(items[0].is_dir);
    }
}
