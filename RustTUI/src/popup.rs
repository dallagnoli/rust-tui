use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::{App, Popup, ASSETS};

pub fn render(frame: &mut Frame, app: &App) {
    match &app.popup {
        Popup::None => {}
        Popup::Preview { scroll } => render_preview(frame, app, *scroll),
        Popup::Confirm => render_confirm(frame, app),
        Popup::Running => render_running(frame),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn render_preview(frame: &mut Frame, app: &App, scroll: u16) {
    let area = centered_rect(70, 70, frame.area());

    let script = match app.selected_script() {
        Some(s) => s,
        None => return,
    };

    let asset_path = format!(
        "{}/{}",
        app.categories[app.selected_category].folder,
        script.file
    );

    let content = ASSETS
        .get_file(&asset_path)
        .and_then(|f| f.contents_utf8())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Could not read file: {}", script.file));

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(content)
            .block(Block::default()
                .title(format!(" Preview: {} ", script.name))
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        area,
    );
}

fn render_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());

    let script = match app.selected_script() {
        Some(s) => s,
        None => return,
    };

    let text = format!(
        "\n  {}\n\n  {}\n\n  Press y to run, n to cancel.",
        script.name, script.description
    );

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default()
                .title(" Run Script? ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_running(frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new("\n  Execution output will appear here...")
            .block(Block::default()
                .title(" Running ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .style(Style::default().fg(Color::White)),
        area,
    );
}
