mod ui;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io;

pub enum Focus {
    Sidebar,
    MainList,
}

pub struct Category {
    pub name: String,
    pub items: Vec<String>,
}

pub struct App {
    pub categories: Vec<Category>,
    pub selected_category: usize,
    pub category_state: ListState, // Tracks blue box in sidebar
    pub item_state: ListState,     // Tracks blue box in main list
    pub focus: Focus,
}

impl App {
    fn new() -> Self {
        let categories = vec![
            Category {
                name: "Files".into(),
                items: vec!["main.rs".into(), "ui.rs".into(), "Cargo.toml".into()],
            },
            Category {
                name: "Network".into(),
                items: vec!["WiFi".into(), "Ethernet".into(), "Bluetooth".into()],
            },
            Category {
                name: "System".into(),
                items: vec!["CPU Usage".into(), "Memory".into(), "Processes".into()],
            },
        ];
        
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
        self.selected_category = (self.selected_category + 1) % self.categories.len();
        self.category_state.select(Some(self.selected_category));
        self.item_state.select(Some(0));
    }

    pub fn prev_category(&mut self) {
        if self.selected_category == 0 {
            self.selected_category = self.categories.len() - 1;
        } else {
            self.selected_category -= 1;
        }
        self.category_state.select(Some(self.selected_category));
        self.item_state.select(Some(0));
    }

    pub fn next_item(&mut self) {
        let i = match self.item_state.selected() {
            Some(i) => {
                if i >= self.categories[self.selected_category].items.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.item_state.select(Some(i));
    }

    pub fn prev_item(&mut self) {
        let i = match self.item_state.selected() {
            Some(i) => {
                if i == 0 { self.categories[self.selected_category].items.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.item_state.select(Some(i));
    }
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
