use crate::app::{App, AppState};
use crate::scanner::format_size;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_file_list(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = match app.state {
        AppState::Scanning => "Scanning...",
        AppState::Selecting => {
            let total_size = app.get_selected_size();
            let count = app.get_selected_count();
            &format!("Selected: {} items ({})", count, format_size(total_size))
        }
        AppState::Confirming => "Press Enter to confirm deletion",
        AppState::Cleaning => {
            &format!("Cleaning... {}/{}", app.clean_progress, app.clean_total)
        }
        AppState::Complete => "Complete!",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Trash Cleaner");

    let paragraph = Paragraph::new(header)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let checkbox = if item.selected { "[x]" } else { "[ ]" };
            let path = item.path.display().to_string();
            let size = item.size_str();
            let category = &item.category;

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", checkbox), style),
                Span::styled(format!("{:<15} ", category), style),
                Span::styled(format!("{:<60} ", path), style),
                Span::styled(size, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Files");

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (disk_before, disk_after) = match (&app.disk_before, &app.disk_after) {
        (Some(before), Some(after)) => (before.available_str(), after.available_str()),
        (Some(before), None) => (before.available_str(), "-".to_string()),
        _ => ("-".to_string(), "-".to_string()),
    };

    let status = match app.state {
        AppState::Complete => {
            format!(
                "Disk: {} free → {} free (+{} freed)",
                disk_before,
                disk_after,
                app.get_disk_freed_str()
            )
        }
        _ => {
            format!("Disk: {} free", disk_before)
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Disk Space");

    let paragraph = Paragraph::new(status)
        .block(block)
        .style(Style::default().fg(Color::Green));

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.state {
        AppState::Scanning => vec![
            Span::styled("Scanning... Please wait", Style::default().fg(Color::Yellow)),
        ],
        AppState::Selecting => vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Move  "),
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(" Toggle  "),
            Span::styled("A", Style::default().fg(Color::Cyan)),
            Span::raw(" Select All  "),
            Span::styled("N", Style::default().fg(Color::Cyan)),
            Span::raw(" Deselect All  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Confirm  "),
            Span::styled("Q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit"),
        ],
        AppState::Confirming => vec![
            Span::styled("Enter", Style::default().fg(Color::Red)),
            Span::raw(" Delete  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Cancel"),
        ],
        AppState::Cleaning => vec![
            Span::styled("Deleting files...", Style::default().fg(Color::Yellow)),
        ],
        AppState::Complete => vec![
            Span::styled("Q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit  "),
            Span::styled("R", Style::default().fg(Color::Cyan)),
            Span::raw(" Rescan"),
        ],
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Controls");

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
        .block(Block::default().title("Cleaning...").borders(Borders::ALL))
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
