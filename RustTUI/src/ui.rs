use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{App, Focus};

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme_active = Color::Blue;
    let theme_inactive = Color::White;
    let theme_title = Color::Yellow;

    let root_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(root_chunks[0]);

    // --- SIDEBAR ---
    let is_sidebar_focused = matches!(app.focus, Focus::Sidebar);
    let sidebar_border_color = if is_sidebar_focused { theme_active } else { theme_inactive };

    let category_names: Vec<ListItem> = app.categories
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let is_selected = i == app.selected_category;
            let mut style = Style::default().fg(Color::White);
            if is_selected {
                style = style.fg(theme_active).add_modifier(Modifier::BOLD);
                if is_sidebar_focused {
                    style = style.fg(Color::Black);
                }
            }
            ListItem::new(c.name.as_str()).style(style)
        })
        .collect();

    let mut sidebar_list = List::new(category_names)
        .block(Block::default()
            .title(" Categories ")
            .title_style(Style::default().fg(theme_title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(sidebar_border_color)))
        .highlight_symbol(">> ");

    if is_sidebar_focused {
        sidebar_list = sidebar_list.highlight_style(Style::default().bg(theme_active));
    }

    frame.render_stateful_widget(sidebar_list, main_chunks[0], &mut app.category_state);

    // --- MAIN LIST ---
    let is_list_focused = matches!(app.focus, Focus::MainList);
    let list_border_color = if is_list_focused { theme_active } else { theme_inactive };

    let current_cat = &app.categories[app.selected_category];
    let selected_item = app.item_state.selected().unwrap_or(0);
    let items: Vec<ListItem> = current_cat.scripts
        .iter()
        .enumerate()
        .map(|(i, script)| {
            let mut style = Style::default().fg(Color::White);
            if i == selected_item {
                style = style.fg(theme_active).add_modifier(Modifier::BOLD);
                if is_list_focused {
                    style = style.fg(Color::Black);
                }
            }
            ListItem::new(script.name.as_str()).style(style)
        })
        .collect();

    let mut main_list = List::new(items)
        .block(Block::default()
            .title(format!(" {} ", current_cat.name))
            .title_style(Style::default().fg(theme_title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(list_border_color)))
        .highlight_symbol("> ");

    if is_list_focused {
        main_list = main_list.highlight_style(
            Style::default()
                .bg(theme_active)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    }

    frame.render_stateful_widget(main_list, main_chunks[1], &mut app.item_state);

    // --- FOOTER ---
    let help_text = match app.focus {
        Focus::Sidebar => "Navigate: ↑/↓ | Focus List: → | Switch Tab: Tab | Quit: q",
        Focus::MainList => "Select: ↑/↓ | Preview: P | Run: Enter | Focus Sidebar: ← | Quit: q",
    };

    let footer = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Keys ")
            .title_style(Style::default().fg(theme_title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_inactive)));

    frame.render_widget(footer, root_chunks[1]);

    // --- POPUPS ---
    crate::popup::render(frame, app);
}
