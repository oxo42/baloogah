use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &App) {
    let constraints = if app.errors.is_empty() {
        vec![Constraint::Length(3), Constraint::Min(0)]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(5),
        ]
    };

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

    if !app.errors.is_empty() {
        let error_items: Vec<Line> = app
            .errors
            .iter()
            .map(|e| Line::from(format!("Error: {}", e)))
            .collect();
        let error_block = Paragraph::new(error_items)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().title("Errors").borders(Borders::ALL));
        f.render_widget(error_block, chunks[2]);
    }
}
