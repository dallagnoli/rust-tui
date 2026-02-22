mod ui;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use serde::Deserialize;
use std::{fs, io, path::PathBuf};

pub enum Focus {
    Sidebar,
    MainList,
}

#[derive(Deserialize)]
struct Tab {
    name: String,
    folder: String,
}

#[derive(Deserialize)]
struct TabsFile {
    tab: Vec<Tab>,
}

#[derive(Deserialize, Clone)]
pub struct Script {
    pub name: String,
    pub description: String,
    pub file: String,
}

#[derive(Deserialize)]
struct TabDataFile {
    script: Vec<Script>,
}

pub struct Category {
    pub name: String,
    pub scripts: Vec<Script>,
}

pub struct App {
    pub categories: Vec<Category>,
    pub selected_category: usize,
    pub category_state: ListState,
    pub item_state: ListState,
    pub focus: Focus,
}

impl App {
    fn new() -> Self {
        let categories = load_categories().unwrap_or_else(|e| {
            eprintln!("Failed to load scripts: {e}");
            vec![]
        });

        let mut category_state = ListState::default();
        category_state.select(Some(0));

        let mut item_state = ListState::default();
        item_state.select(Some(0));

        Self {
            categories,
            selected_category: 0,
            category_state,
            item_state,
            focus: Focus::Sidebar,
        }
    }

    pub fn next_category(&mut self) {
        if self.categories.is_empty() { return; }
        self.selected_category = (self.selected_category + 1) % self.categories.len();
        self.category_state.select(Some(self.selected_category));
        self.item_state.select(Some(0));
    }

    pub fn prev_category(&mut self) {
        if self.categories.is_empty() { return; }
        if self.selected_category == 0 {
            self.selected_category = self.categories.len() - 1;
        } else {
            self.selected_category -= 1;
        }
        self.category_state.select(Some(self.selected_category));
        self.item_state.select(Some(0));
    }

    pub fn next_item(&mut self) {
        let len = self.categories[self.selected_category].scripts.len();
        if len == 0 { return; }
        let i = match self.item_state.selected() {
            Some(i) => if i >= len - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.item_state.select(Some(i));
    }

    pub fn prev_item(&mut self) {
        let len = self.categories[self.selected_category].scripts.len();
        if len == 0 { return; }
        let i = match self.item_state.selected() {
            Some(i) => if i == 0 { len - 1 } else { i - 1 },
            None => 0,
        };
        self.item_state.select(Some(i));
    }
}

fn load_categories() -> Result<Vec<Category>, Box<dyn std::error::Error>> {
    let base = PathBuf::from("../scripts");

    let tabs_raw = fs::read_to_string(base.join("tabs.toml"))?;
    let tabs_file: TabsFile = toml::from_str(&tabs_raw)?;

    let mut categories = Vec::new();

    for tab in tabs_file.tab {
        let tab_data_path = base.join(&tab.folder).join("tab_data.toml");
        let tab_data_raw = fs::read_to_string(&tab_data_path)?;
        let tab_data: TabDataFile = toml::from_str(&tab_data_raw)?;

        categories.push(Category {
            name: tab.name,
            scripts: tab_data.script,
        });
    }

    Ok(categories)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => app.next_category(),
                        KeyCode::BackTab => app.prev_category(),
                        KeyCode::Left => app.focus = Focus::Sidebar,
                        KeyCode::Right => app.focus = Focus::MainList,
                        KeyCode::Down => match app.focus {
                            Focus::Sidebar => app.next_category(),
                            Focus::MainList => app.next_item(),
                        },
                        KeyCode::Up => match app.focus {
                            Focus::Sidebar => app.prev_category(),
                            Focus::MainList => app.prev_item(),
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
