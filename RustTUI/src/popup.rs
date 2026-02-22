use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};
use crate::{App, Popup, PreviewCache, ASSETS};

const HIGHLIGHT_NAMES: &[&str] = &[
    "comment",
    "constant",
    "constant.builtin",
    "function",
    "function.builtin",
    "keyword",
    "operator",
    "punctuation",
    "string",
    "type",
    "variable",
    "variable.builtin",
];

fn highlight_color(name: &str) -> Color {
    match name {
        "comment"          => Color::Rgb(98, 114, 164),
        "constant"         => Color::Rgb(255, 184, 108),
        "constant.builtin" => Color::Rgb(255, 184, 108),
        "function"         => Color::Rgb(80, 250, 123),
        "function.builtin" => Color::Rgb(80, 250, 123),
        "keyword"          => Color::Rgb(255, 121, 198),
        "operator"         => Color::Rgb(248, 248, 242),
        "punctuation"      => Color::Rgb(248, 248, 242),
        "string"           => Color::Rgb(241, 250, 140),
        "type"             => Color::Rgb(139, 233, 253),
        "variable"         => Color::Rgb(248, 248, 242),
        "variable.builtin" => Color::Rgb(189, 147, 249),
        _                  => Color::White,
    }
}

pub fn build_highlighted_lines(content: &str) -> Vec<Line<'static>> {
    let mut highlighter = Highlighter::new();

    let language = tree_sitter_bash::LANGUAGE.into();
    let mut config = HighlightConfiguration::new(
        language,
        "bash",
        tree_sitter_bash::HIGHLIGHT_QUERY,
        "",
        "",
    ).expect("Failed to build highlight config");
    config.configure(HIGHLIGHT_NAMES);

    let source = content.as_bytes();
    let events = highlighter.highlight(&config, source, None, |_| None);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut color_stack: Vec<Color> = Vec::new();

    let events = match events {
        Ok(e) => e,
        Err(_) => {
            return content
                .lines()
                .map(|l| Line::from(Span::styled(
                    l.to_string(),
                    Style::default().fg(Color::White),
                )))
                .collect();
        }
    };

    for event in events.flatten() {
        match event {
            HighlightEvent::HighlightStart(h) => {
                let color = highlight_color(HIGHLIGHT_NAMES[h.0]);
                color_stack.push(color);
            }
            HighlightEvent::HighlightEnd => {
                color_stack.pop();
            }
            HighlightEvent::Source { start, end } => {
                let slice = &content[start..end];
                let color = color_stack.last().copied().unwrap_or(Color::White);

                for (i, part) in slice.split('\n').enumerate() {
                    if i > 0 {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                    if !part.is_empty() {
                        current_spans.push(Span::styled(
                            part.to_string(),
                            Style::default().fg(color),
                        ));
                    }
                }
            }
        }
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    lines
}

pub fn render(frame: &mut Frame, app: &mut App) {
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

fn render_preview(frame: &mut Frame, app: &mut App, scroll: u16) {
    let area = centered_rect(70, 70, frame.area());

    let script = match app.selected_script() {
        Some(s) => s.clone(),
        None => return,
    };

    let asset_path = format!(
        "{}/{}",
        app.categories[app.selected_category].folder,
        script.file
    );

    let needs_update = app.preview_cache
        .as_ref()
        .map(|c| c.asset_path != asset_path)
        .unwrap_or(true);

    if needs_update {
        let content = ASSETS
            .get_file(&asset_path)
            .and_then(|f| f.contents_utf8())
            .unwrap_or("Could not read file.");

        let lines = build_highlighted_lines(content);
        app.preview_cache = Some(PreviewCache {
            asset_path,
            lines,
        });
    }

    let lines = match &app.preview_cache {
        Some(c) => c.lines.clone(),
        None => return,
    };

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title(format!(" Preview: {} ", script.name))
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
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
