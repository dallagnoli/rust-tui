use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{App, Focus};

pub fn render(frame: &mut Frame, app: &mut App) {
    // 1. Root Layout: Split vertically for the Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content area
            Constraint::Length(1), // Footer help bar
        ])
        .split(frame.area());

    // 2. Content Layout: Split horizontally for Sidebar/Main List
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), 
            Constraint::Percentage(70)
        ])
        .split(chunks[0]);

    // --- SIDEBAR RENDERING ---
    let sidebar_border_style = match app.focus {
        Focus::Sidebar => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::DarkGray),
    };

    let category_names: Vec<ListItem> = app.categories
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut style = Style::default();
            if i == app.selected_category {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                if let Focus::Sidebar = app.focus {
                    style = style.bg(Color::Rgb(50, 50, 50)); 
                }
            }
            ListItem::new(c.name.as_str()).style(style)
        })
        .collect();

    let sidebar = List::new(category_names)
        .block(Block::default()
            .title(" Categories ")
            .borders(Borders::ALL)
            .border_style(sidebar_border_style));
    frame.render_widget(sidebar, main_layout[0]);

    // --- MAIN LIST RENDERING ---
    let main_border_style = match app.focus {
        Focus::MainList => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::DarkGray),
    };

    let current_cat = &app.categories[app.selected_category];
    let items: Vec<ListItem> = current_cat.items
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(format!(" {} ", current_cat.name))
            .borders(Borders::ALL)
            .border_style(main_border_style))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, main_layout[1], &mut app.item_state);

    // --- FOOTER RENDERING ---
    let help_menu = match app.focus {
        Focus::Sidebar => " Navigate: ↑/↓ | Enter List: → | Switch Category: Tab | Quit: q ",
        Focus::MainList => " Select Item: ↑/↓ | Back to Sidebar: ← | Switch Category: Tab | Quit: q ",
    };

    let footer = Paragraph::new(help_menu)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    frame.render_widget(footer, chunks[1]);
}
