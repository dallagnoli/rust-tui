mod ui;
mod popup;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use include_dir::{include_dir, Dir};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use serde::Deserialize;
use std::io;

pub static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../scripts");

pub enum Focus {
    Sidebar,
    MainList,
}

pub enum Popup {
    None,
    Preview { scroll: u16 },
    Confirm,
    Running,
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
    pub folder: String,
    pub scripts: Vec<Script>,
}

pub struct App {
    pub categories: Vec<Category>,
    pub selected_category: usize,
    pub category_state: ListState,
    pub item_state: ListState,
    pub focus: Focus,
    pub popup: Popup,
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
            popup: Popup::None,
        }
    }

    pub fn selected_script(&self) -> Option<&Script> {
        let i = self.item_state.selected()?;
        self.categories[self.selected_category].scripts.get(i)
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

fn read_asset_str(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = ASSETS.get_file(path).ok_or_else(|| format!("Asset not found: {path}"))?;
    let text = file.contents_utf8().ok_or("File is not valid UTF-8")?;
    Ok(text.to_string())
}

fn load_categories() -> Result<Vec<Category>, Box<dyn std::error::Error>> {
    let tabs_raw = read_asset_str("tabs.toml")?;
    let tabs_file: TabsFile = toml::from_str(&tabs_raw)?;

    let mut categories = Vec::new();

    for tab in tabs_file.tab {
        let tab_data_raw = read_asset_str(&format!("{}/tab_data.toml", tab.folder))?;
        let tab_data: TabDataFile = toml::from_str(&tab_data_raw)?;

        categories.push(Category {
            name: tab.name,
            folder: tab.folder,
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
                    match &app.popup {
                        Popup::None => match key.code {
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
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                if matches!(app.focus, Focus::MainList) {
                                    if app.selected_script().is_some() {
                                        app.popup = Popup::Preview { scroll: 0 };
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if matches!(app.focus, Focus::MainList) {
                                    if app.selected_script().is_some() {
                                        app.popup = Popup::Confirm;
                                    }
                                }
                            }
                            _ => {}
                        },
                        Popup::Preview { scroll } => {
                            let current_scroll = *scroll;
                            match key.code {
                                KeyCode::Esc => app.popup = Popup::None,
                                KeyCode::Down => app.popup = Popup::Preview { scroll: current_scroll + 1 },
                                KeyCode::Up => app.popup = Popup::Preview { scroll: current_scroll.saturating_sub(1) },
                                _ => {}
                            }
                        }
                        Popup::Confirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.popup = Popup::Running,
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.popup = Popup::None,
                            _ => {}
                        },
                        Popup::Running => match key.code {
                            KeyCode::Esc => app.popup = Popup::None,
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
