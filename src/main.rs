use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

mod app;
mod disk;
mod error;
mod scanner;
mod targets;
mod tree;
mod ui;

use app::{App, AppState};

fn main() -> Result<()> {
    // 设置 panic hook 以确保在程序崩溃时恢复终端
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        let _ = execute!(stdout, crossterm::cursor::Show);
        default_panic(info);
    }));

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // 无论运行结果如何，都恢复终端状态
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    app.scan()?;
    let mut last_g_press: Option<std::time::Instant> = None;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // 处理自动状态转换
        match app.state {
            AppState::Scanning => {
                app.update_scan()?;
            }
            AppState::Cleaning => {
                let done = app.clean_next()?;
                if done {
                    app.finish_clean()?;
                }
            }
            _ => {}
        }

        if event::poll(std::time::Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match app.state {
                AppState::Scanning => {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
                AppState::Selecting | AppState::Complete => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => {
                        app.error_message = None;
                        last_g_press = None;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app = App::new();
                        app.scan()?;
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Char(' ') => app.toggle_selected(),
                    KeyCode::Char('a') | KeyCode::Char('A') => app.select_all(),
                    KeyCode::Char('d') | KeyCode::Char('D') if app.get_selected_count() > 0 => {
                        app.state = AppState::Confirming;
                    }
                    KeyCode::Char('g') => {
                        let now = std::time::Instant::now();
                        if let Some(last) = last_g_press {
                            if now.duration_since(last).as_millis() < 500 {
                                app.selected_index = 0;
                                last_g_press = None;
                            } else {
                                last_g_press = Some(now);
                            }
                        } else {
                            last_g_press = Some(now);
                        }
                    }
                    KeyCode::Char('G') => {
                        let now = std::time::Instant::now();
                        if let Some(last) = last_g_press {
                            if now.duration_since(last).as_millis() < 500 {
                                if !app.display_items.is_empty() {
                                    app.selected_index = app.display_items.len() - 1;
                                }
                                last_g_press = None;
                            } else {
                                last_g_press = Some(now);
                            }
                        } else {
                            last_g_press = Some(now);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(item) = app.display_items.get(app.selected_index)
                            && item.is_dir
                        {
                            app.toggle_collapse();
                        }
                    }
                    _ => {
                        last_g_press = None;
                    }
                },
                AppState::Confirming => match key.code {
                    KeyCode::Enter | KeyCode::Char('d') | KeyCode::Char('D') => {
                        app.start_clean();
                    }
                    KeyCode::Esc => {
                        app.state = AppState::Selecting;
                    }
                    _ => {}
                },
                AppState::Cleaning => {
                    // 清理状态下暂不响应其他按键
                }
            }
        }
    }
}
