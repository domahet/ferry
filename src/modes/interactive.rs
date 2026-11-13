use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal,
};
use crate::config::Config;

const HELP: &str = "↑/↓ j/k move · PgUp/PgDn · Home/End · Space toggle · a all/none · Enter confirm · q/Esc cancel";

struct Entry {
    path: PathBuf,
    label: String,
}

struct Picker {
    entries: Vec<Entry>,
    selected: BTreeSet<usize>,
    cursor: usize,
    list_state: ListState,
    page_height: usize,
}

impl Picker {
    fn new(entries: Vec<Entry>) -> Self {
        Picker {
            entries,
            selected: BTreeSet::new(),
            cursor: 0,
            list_state: ListState::default().with_selected(Some(0)),
            page_height: 1,
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        let last = self.entries.len().saturating_sub(1);
        self.cursor = self.cursor.saturating_add_signed(delta).min(last);
        self.list_state.select(Some(self.cursor));
    }

    fn toggle(&mut self) {
        if !self.selected.remove(&self.cursor) {
            self.selected.insert(self.cursor);
        }
    }

    fn toggle_all(&mut self) {
        if self.selected.len() == self.entries.len() {
            self.selected.clear();
        } else {
            self.selected = (0..self.entries.len()).collect();
        }
    }

    fn selected_paths(&self) -> Vec<PathBuf> {
        self.selected.iter().map(|&i| self.entries[i].path.clone()).collect()
    }
}

/// Lets the user pick files and directories directly inside `start_path`.
/// Returns `None` if the picker was cancelled.
pub fn run_tui_selection(start_path: &Path, config: &Config) -> Result<Option<Vec<PathBuf>>, String> {
    // Selections are consumed from other directories, so they must be absolute.
    let start_path = start_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve '{}': {}", start_path.display(), e))?;

    let entries = list_entries(&start_path)?;
    if entries.is_empty() {
        config.print_normal(&format!("Nothing to select in '{}'.", start_path.display()));
        return Ok(Some(Vec::new()));
    }

    // try_init also installs a panic hook that restores the terminal.
    let mut terminal = ratatui::try_init().map_err(|e| format!("Failed to start the terminal UI: {}", e))?;
    let mut picker = Picker::new(entries);
    let result = run(&mut terminal, &mut picker, &start_path);
    ratatui::try_restore().map_err(|e| format!("Failed to restore the terminal: {}", e))?;

    match result {
        Ok(true) => Ok(Some(picker.selected_paths())),
        Ok(false) => Ok(None),
        Err(e) => Err(format!("Terminal UI error: {}", e)),
    }
}

fn list_entries(start_path: &Path) -> Result<Vec<Entry>, String> {
    let read_dir = std::fs::read_dir(start_path)
        .map_err(|e| format!("Failed to read '{}': {}", start_path.display(), e))?;

    let mut entries: Vec<(bool, Entry)> = read_dir
        .filter_map(|e| e.ok())
        .map(|e| {
            let is_dir = e.path().is_dir();
            let mut label = e.file_name().to_string_lossy().into_owned();
            if is_dir {
                label.push('/');
            }
            (is_dir, Entry { path: e.path(), label })
        })
        .collect();
    // Directories first, then case-insensitive by name.
    entries.sort_by_cached_key(|(is_dir, entry)| (!is_dir, entry.label.to_lowercase()));
    Ok(entries.into_iter().map(|(_, entry)| entry).collect())
}

/// Returns Ok(true) when confirmed, Ok(false) when cancelled.
fn run(terminal: &mut DefaultTerminal, picker: &mut Picker, start_path: &Path) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| {
            let [list_area, help_area] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(frame.area());
            picker.page_height = list_area.height.saturating_sub(2).max(1) as usize;

            let items: Vec<ListItem> = picker
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let mark = if picker.selected.contains(&i) { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{} {}", mark, entry.label))
                })
                .collect();

            let title = format!(
                " {} — {} of {} selected ",
                start_path.display(),
                picker.selected.len(),
                picker.entries.len()
            );
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(Line::from(title)))
                .highlight_symbol("> ")
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            frame.render_stateful_widget(list, list_area, &mut picker.list_state);
            frame.render_widget(Paragraph::new(HELP), help_area);
        })?;

        let Event::Key(key) = event::read()? else { continue };
        // Windows also reports key releases; act on presses only.
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let page = picker.page_height as isize;
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(false),
            KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
            KeyCode::Enter => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => picker.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => picker.move_cursor(1),
            KeyCode::PageUp => picker.move_cursor(-page),
            KeyCode::PageDown => picker.move_cursor(page),
            KeyCode::Home | KeyCode::Char('g') => picker.move_cursor(isize::MIN),
            KeyCode::End | KeyCode::Char('G') => picker.move_cursor(isize::MAX),
            KeyCode::Char(' ') => picker.toggle(),
            KeyCode::Char('a') => picker.toggle_all(),
            _ => {}
        }
    }
}
