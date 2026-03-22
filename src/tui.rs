use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn shorten_name(name: &str) -> &str {
    name.split('/').last().unwrap_or(name)
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        return name.to_string();
    }
    let side_len = (max_len - 3) / 2;
    let start = &name[..side_len];
    let end = &name[name.len() - side_len..];
    format!("{}...{}", start, end)
}

pub fn ui(f: &mut Frame, app: &App) {
    let finished = app.completed_images >= app.total_images;
    
    let mut constraints = vec![Constraint::Length(3), Constraint::Min(0)];
    if !app.errors.is_empty() {
        constraints.push(Constraint::Length(5));
    }
    if finished {
        constraints.push(Constraint::Length(1));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice())
        .split(f.area());

    let title_text = format!(
        "Updating docker images ({}/{})",
        app.completed_images, app.total_images
    );
    let title = Paragraph::new(title_text).style(Style::default().fg(Color::White));
    f.render_widget(title, chunks[0]);

    let area = chunks[1];
    let width = area.width as usize;
    // Account for borders (2)
    let available_width = width.saturating_sub(2);

    let mut list_items = Vec::new();
    let spinner_char = if finished {
        "✓"
    } else {
        SPINNER_CHARS[(app.tick as usize) % SPINNER_CHARS.len()]
    };

    // Calculate dynamic bar width
    // max 30, but let's say it takes up to 30% of the screen but at least 10 and at most 30
    let bar_width = (available_width / 3).clamp(10, 30);
    
    // Remaining width for the name: available_width - bar_width - spacing - spinner
    // space: 1 before [, 1 before spinner
    let name_max_width = available_width.saturating_sub(bar_width + 4);

    for img in &app.image_order {
        let short_name = shorten_name(img);
        let display_name = truncate_name(short_name, name_max_width);
        
        let progress = app.image_progress.get(img).copied().unwrap_or(0.0);
        let has_error = app.image_errors.get(img).map(|e| !e.is_empty()).unwrap_or(false);
        
        let inner_bar_len = bar_width.saturating_sub(2); // [] borders
        let filled = (progress * inner_bar_len as f32).round() as usize;
        let filled = filled.min(inner_bar_len);
        let empty = inner_bar_len - filled;

        let bar = format!("[{}{}]", "X".repeat(filled), ".".repeat(empty));
        
        // Final line format: Name <flexible space> [Progress] Spinner
        let line_str = format!(
            "{:<name_width$} {} {}",
            display_name,
            bar,
            spinner_char,
            name_width = name_max_width
        );

        let style = if has_error {
            Style::default().fg(Color::Red)
        } else if progress >= 1.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        
        list_items.push(Line::from(line_str).style(style));
    }

    let list_block = Paragraph::new(list_items)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(list_block, chunks[1]);

    let mut next_chunk = 2;
    if !app.errors.is_empty() {
        let error_items: Vec<Line> = app
            .errors
            .iter()
            .map(|e| Line::from(format!("Error: {}", e)))
            .collect();
        let error_block = Paragraph::new(error_items)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().title("Errors").borders(Borders::ALL));
        f.render_widget(error_block, chunks[next_chunk]);
        next_chunk += 1;
    }

    if finished {
        let footer = Paragraph::new("Press any key to exit").style(Style::default().fg(Color::Yellow));
        f.render_widget(footer, chunks[next_chunk]);
    }
}
