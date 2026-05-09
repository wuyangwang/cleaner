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
    let (disk_before, disk_after) = match (&app.disk_before, &app.disk_after) {
        (Some(before), Some(after)) => (before.available_str(), after.available_str()),
        (Some(before), None) => (before.available_str(), "-".to_string()),
        _ => ("-".to_string(), "-".to_string()),
    };

    let disk_info = match app.state {
        AppState::Complete => format!(
            " | 磁盘: {} → {} (+{})",
            disk_before,
            disk_after,
            app.get_disk_freed_str()
        ),
        _ => format!(" | 磁盘: {} 可用", disk_before),
    };

    let (state_text, state_style) = match app.state {
        AppState::Scanning => (
            "正在扫描...".to_string(),
            Style::default().fg(Color::Yellow),
        ),
        AppState::Selecting => {
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
        AppState::Confirming => (
            "按回车确认删除".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        AppState::Cleaning => (
            format!("正在删除... {}/{}", app.clean_progress, app.clean_total),
            Style::default().fg(Color::Yellow),
        ),
        AppState::Complete => (
            "清理完成!".to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
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
                    Span::styled(
                        format!("{}{}{} ", indent, collapse_icon, dir_path),
                        row_style,
                    ),
                    Span::styled(info, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            } else {
                let checkbox = if item.selected { "[✓]" } else { "[ ]" };
                let indent = "  ".repeat(item.depth);
                let file_name = item
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| item.path.display().to_string());
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
                    Span::styled(format!("{:<60} ", file_name), row_style),
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
    let help = match app.state {
        AppState::Scanning => vec![Span::styled(
            "正在扫描... 请稍候",
            Style::default().fg(Color::Yellow),
        )],
        AppState::Selecting => vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::styled(" 移动  ", Style::default().fg(Color::Gray)),
            Span::styled("空格", Style::default().fg(Color::Cyan)),
            Span::styled(" 折叠/选择  ", Style::default().fg(Color::Gray)),
            Span::styled("A", Style::default().fg(Color::Cyan)),
            Span::styled(" 全选  ", Style::default().fg(Color::Gray)),
            Span::styled("N", Style::default().fg(Color::Cyan)),
            Span::styled(" 取消全选  ", Style::default().fg(Color::Gray)),
            Span::styled("回车", Style::default().fg(Color::Green)),
            Span::styled(" 确认删除  ", Style::default().fg(Color::Gray)),
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::styled(" 退出", Style::default().fg(Color::Gray)),
        ],
        AppState::Confirming => vec![
            Span::styled(
                "回车",
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
        AppState::Complete => vec![
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::styled(" 退出  ", Style::default().fg(Color::Gray)),
            Span::styled("R", Style::default().fg(Color::Cyan)),
            Span::styled(" 重新扫描", Style::default().fg(Color::Gray)),
        ],
    };

    let block = Block::default().borders(Borders::ALL).title("操作说明");

    let paragraph = Paragraph::new(Line::from(help)).block(block);

    f.render_widget(paragraph, area);
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
