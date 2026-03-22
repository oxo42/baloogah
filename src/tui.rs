use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

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

    let mut list_items = Vec::new();

    for img in &app.image_order {
        let progress = app.image_progress.get(img).copied().unwrap_or(0.0);
        let bar_len = 20;
        let filled = (progress * bar_len as f32).round() as usize;
        let filled = filled.min(bar_len);
        let empty = bar_len - filled;

        let bar = format!("{}{}", "X".repeat(filled), ".".repeat(empty));
        list_items.push(Line::from(format!("{}: {}", img, bar)));
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
