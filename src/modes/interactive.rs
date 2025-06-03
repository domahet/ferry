use std::path::{Path, PathBuf};
use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{ListItem, List, Block, Borders},
    Terminal,
};
use walkdir::WalkDir;
use crate::config::Config;

struct TuiState {
    items: Vec<PathBuf>,
    selected_indices: Vec<usize>,
    current_scroll_index: usize,
    scroll_offset: usize,
    terminal_height: usize,
}

impl TuiState {
    fn new(items: Vec<PathBuf>, terminal_height: u16) -> Self {
        TuiState {
            items,
            selected_indices: Vec::new(),
            current_scroll_index: 0,
            scroll_offset: 0,
            terminal_height: terminal_height as usize,
        }
    }

    fn toggle_selection(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let index = self.current_scroll_index;
        if let Some(pos) = self.selected_indices.iter().position(|&i| i == index) {
            self.selected_indices.remove(pos);
        } else {
            self.selected_indices.push(index);
            self.selected_indices.sort_unstable();
        }
    }

    fn move_up(&mut self) {
        if self.current_scroll_index > 0 {
            self.current_scroll_index -= 1;
            if self.current_scroll_index < self.scroll_offset {
                self.scroll_offset -= 1;
            }
        }
    }

    fn move_down(&mut self) {
        if self.current_scroll_index < self.items.len().saturating_sub(1) {
            self.current_scroll_index += 1;
            let visible_height = self.terminal_height.saturating_sub(2);
            if self.current_scroll_index >= self.scroll_offset + visible_height {
                self.scroll_offset += 1;
            }
        }
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.selected_indices.iter()
            .filter_map(|&i| self.items.get(i).cloned())
            .collect()
    }
}

pub fn run_tui_selection(start_path: &Path, config: &Config) -> Result<Vec<PathBuf>, String> {

    let all_files = WalkDir::new(start_path)
        .max_depth(1) 
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path() != start_path) 
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<PathBuf>>();

    if all_files.is_empty() {
        config.print_normal(&format!("No files found in '{}' for TUI selection.", start_path.display()));
        return Ok(Vec::new());
    }

    // Setup terminal
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| format!("Failed to enter alternate screen: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("Failed to create terminal: {}", e))?;

    let initial_height = terminal.size().map_err(|e| format!("Failed to get terminal size: {}", e))?.height;
    let mut app_state = TuiState::new(all_files, initial_height);

    let result = run_app(&mut terminal, &mut app_state);

    // Restore terminal
    disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| format!("Failed to leave alternate screen: {}", e))?;
    terminal.show_cursor().map_err(|e| format!("Failed to show cursor: {}", e))?;

    match result {
        Ok(_) => Ok(app_state.get_selected_paths()),
        Err(e) => Err(e.to_string()),
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app_state: &mut TuiState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();
            app_state.terminal_height = size.height as usize;

            let mut list_items: Vec<ListItem> = Vec::new();
            let visible_height = app_state.terminal_height.saturating_sub(2);
            let visible_start = app_state.scroll_offset;
            let visible_end = (app_state.scroll_offset + visible_height).min(app_state.items.len());

            for i in visible_start..visible_end {
                if let Some(path) = app_state.items.get(i) {
                    let mut s = path.display().to_string();
                    if let Some(stripped_path) = path.strip_prefix(app_state.items[0].parent().unwrap_or(path))
                        .ok().and_then(|p| if p.as_os_str().is_empty() { None } else { Some(p) }) {
                        s = stripped_path.display().to_string();
                    } else { 
                        s = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned();
                    }

                    if app_state.selected_indices.contains(&i) {
                        s = format!("[x] {}", s);
                    } else {
                        s = format!("[ ] {}", s);
                    }
                    if i == app_state.current_scroll_index {
                        s = format!("> {}", s);
                    } else {
                        s = format!("  {}", s);
                    }
                    list_items.push(ListItem::new(s));
                }
            }

            let list_widget = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Select Files (Space: toggle, Enter: confirm, q: quit)"))
                .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

            f.render_widget(list_widget, size);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Up => {
                    app_state.move_up();
                }
                KeyCode::Down => {
                    app_state.move_down();
                }
                KeyCode::Char(' ') => {
                    app_state.toggle_selection();
                }
                KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}