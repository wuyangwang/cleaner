use crate::app::{App, AppState};
use crate::scanner::{format_size, shorten_path};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_file_list(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let disk_before = match &app.disk_before {
        Some(before) => before.available_str(),
        _ => "-".to_string(),
    };

    let disk_info = format!(" | 磁盘: {} 可用", disk_before);

    let (state_text, state_style) = match app.state {
        AppState::Scanning => (
            app.current_scanning.clone(),
            Style::default().fg(Color::Yellow),
        ),
        AppState::Selecting | AppState::Complete => {
            if app.display_items.is_empty() {
                (
                    "所有项目已清理".to_string(),
                    Style::default().fg(Color::Green),
                )
            } else {
                let total_size = app.get_selected_size();
                let count = app.get_selected_count();
                let st = if count > 0 {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                (
                    format!("已选择: {} 项 ({})", count, format_size(total_size)),
                    st,
                )
            }
        }
        AppState::Confirming => (
            "按回车或 D 确认删除".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        AppState::Cleaning => (
            format!("正在删除... {}/{}", app.clean_progress, app.clean_total),
            Style::default().fg(Color::Yellow),
        ),
    };

    let block = Block::default().borders(Borders::ALL).title("垃圾清理");
    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(state_text, state_style),
        Span::styled(disk_info, Style::default().fg(Color::DarkGray)),
    ]))
    .block(block);

    f.render_widget(paragraph, area);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    if matches!(app.state, AppState::Scanning) {
        let block = Block::default().borders(Borders::ALL).title("文件列表");
        let paragraph = Paragraph::new("\n\n  正在扫描系统中，请稍候...\n\n  扫描顺序：\n  1. 浏览器与系统缓存\n  2. 开发工具日志与临时源码\n  3. 包管理下载缓存\n  4. 通用与系统回收站")
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    let total_visual = app.display_items.len();
    let list_height = (area.height.saturating_sub(2)) as usize;

    let offset = if total_visual > list_height {
        std::cmp::min(
            app.selected_index
                .saturating_sub(list_height.saturating_sub(1)),
            total_visual.saturating_sub(list_height),
        )
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .display_items
        .iter()
        .enumerate()
        .skip(offset)
        .take(list_height)
        .map(|(vis_idx, item)| {
            let is_cursor = vis_idx == app.selected_index;

            if item.is_dir {
                let collapse_icon = if item.collapsed { "[+]" } else { "[-]" };

                // 计算目录选择状态图标
                // 由于 flatten_tree 已经处理了 selected 逻辑，我们这里简化判断
                // 我们在 tree.rs 里已经让 selected 为 true 如果有任何子项被选中
                // 为了精确显示半选，我们需要更详细的数据。
                // 暂时用 selected 表示全选/半选，后续细化。

                let dir_node = find_dir_node(&app.tree, &item.path);
                let (checkbox, checkbox_style) = if let Some(node) = dir_node {
                    let sel = node.selected_count();
                    let total = node.file_count();
                    if sel == 0 {
                        ("[ ] ", Style::default().fg(Color::DarkGray))
                    } else if sel == total {
                        (
                            "[✓] ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        (
                            "[-] ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    }
                } else {
                    ("[ ] ", Style::default().fg(Color::DarkGray))
                };

                let dir_path = shorten_path(&item.path);
                let info = format!("({} 项, {})", item.file_count, format_size(item.size));
                let indent = "  ".repeat(item.depth);

                let row_style = if is_cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                };

                let line = Line::from(vec![
                    Span::styled(indent.clone(), row_style),
                    Span::styled(checkbox, checkbox_style),
                    Span::styled(format!("{} {} ", collapse_icon, dir_path), row_style),
                    Span::styled(info, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            } else {
                let checkbox = if item.selected { "[✓]" } else { "[ ]" };
                let indent = "  ".repeat(item.depth);
                let file_path = shorten_path(&item.path);
                let size = format_size(item.size);

                let checkbox_style = if item.selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let row_style = if is_cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if item.selected {
                    Style::default().fg(Color::LightGreen)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(format!("{}{} ", indent, checkbox), checkbox_style),
                    Span::styled(format!("{:<60} ", file_path), row_style),
                    Span::styled(size, row_style),
                ]);
                ListItem::new(line)
            }
        })
        .collect();

    let block = Block::default().borders(Borders::ALL).title("文件列表");

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    if let Some(error_msg) = &app.error_message {
        let error_line = Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(error_msg.as_str(), Style::default().fg(Color::Red)),
            Span::styled("  (按 Esc 关闭)", Style::default().fg(Color::DarkGray)),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("错误信息")
            .border_style(Style::default().fg(Color::Red));
        let paragraph = Paragraph::new(error_line).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let help = match app.state {
        AppState::Scanning => vec![Span::styled(
            "正在扫描... 请稍候",
            Style::default().fg(Color::Yellow),
        )],
        AppState::Selecting | AppState::Complete => vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::styled(" 移动  ", Style::default().fg(Color::Gray)),
            Span::styled("空格", Style::default().fg(Color::Cyan)),
            Span::styled(" 选择  ", Style::default().fg(Color::Gray)),
            Span::styled("回车", Style::default().fg(Color::Cyan)),
            Span::styled(" 折叠  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "D",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 删除  ", Style::default().fg(Color::Gray)),
            Span::styled("A/N", Style::default().fg(Color::Cyan)),
            Span::styled(" 全选/取消  ", Style::default().fg(Color::Gray)),
            Span::styled("R", Style::default().fg(Color::Cyan)),
            Span::styled(" 重新扫描  ", Style::default().fg(Color::Gray)),
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::styled(" 退出", Style::default().fg(Color::Gray)),
        ],
        AppState::Confirming => vec![
            Span::styled(
                "回车/D",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 确认删除  ", Style::default().fg(Color::Red)),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::styled(" 取消", Style::default().fg(Color::Gray)),
        ],
        AppState::Cleaning => vec![Span::styled(
            "正在删除文件...",
            Style::default().fg(Color::Yellow),
        )],
    };

    let block = Block::default().borders(Borders::ALL).title("操作说明");

    let paragraph = Paragraph::new(Line::from(help)).block(block);

    f.render_widget(paragraph, area);
}

fn find_dir_node<'a>(
    nodes: &'a [crate::tree::TreeNode],
    path: &std::path::Path,
) -> Option<&'a crate::tree::DirNode> {
    for node in nodes {
        if let crate::tree::TreeNode::Dir(dir) = node {
            if dir.path == path {
                return Some(dir);
            }
            if let Some(found) = find_dir_node(&dir.children, path) {
                return Some(found);
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn draw_progress(f: &mut Frame, progress: usize, total: usize) {
    let percent = if total > 0 {
        (progress as f64 / total as f64 * 100.0) as u16
    } else {
        0
    };

    let area = f.area();
    let popup = centered_rect(60, 20, area);

    let gauge = Gauge::default()
        .block(Block::default().title("清理中...").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(percent);

    f.render_widget(gauge, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
