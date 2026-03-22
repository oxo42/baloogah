use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, shorten_name, truncate_name};

const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn ui(f: &mut Frame, app: &App) {
    let finished = app.completed_images >= app.total_images;
    
    let mut constraints = vec![Constraint::Length(3), Constraint::Min(0)];
    if !app.errors.is_empty() {
        constraints.push(Constraint::Length(5));
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
    let bar_width = (available_width / 3).clamp(10, 30);
    
    // Remaining width for the name: available_width - bar_width - spacing - spinner
    // space: 1 before [, 1 before spinner
    let name_max_width = available_width.saturating_sub(bar_width + 4);

    for img in &app.image_order {
        let short_name = shorten_name(img);
        let display_name = truncate_name(short_name, name_max_width);
        
        let progress = app.image_progress.get(img).copied().unwrap_or(0.0);
        let has_error = app.image_errors.get(img).map(|e| !e.is_empty()).unwrap_or(false);
        
        let style = if has_error {
            Style::default().fg(Color::Red)
        } else if progress >= 1.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let mut spans = Vec::new();

        // Name
        let name_str = format!("{:<name_max_width$} ", display_name);
        spans.push(Span::styled(name_str, style));

        // Bar start
        spans.push(Span::styled("[", style));

        let inner_bar_len = bar_width.saturating_sub(2); // [] borders
        let filled = (progress * inner_bar_len as f32).round() as usize;
        let filled = filled.min(inner_bar_len);

        for i in 0..inner_bar_len {
            if i < filled {
                if has_error {
                    spans.push(Span::styled("█", Style::default().fg(Color::Red)));
                } else if progress >= 1.0 {
                    spans.push(Span::styled("█", Style::default().fg(Color::Green)));
                } else {
                    // Gradient: Blue (0, 0, 255) to Cyan (0, 255, 255)
                    let ratio = i as f32 / inner_bar_len.saturating_sub(1).max(1) as f32;
                    let g = (ratio * 255.0) as u8;
                    let color = Color::Rgb(0, g, 255);
                    spans.push(Span::styled("█", Style::default().fg(color)));
                }
            } else {
                spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
            }
        }

        // Bar end
        spans.push(Span::styled("] ", style));
        
        // Spinner
        spans.push(Span::styled(spinner_char, style));
        
        list_items.push(Line::from(spans));
    }

    let list_block = Paragraph::new(list_items)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(list_block, chunks[1]);

    let next_chunk = 2;
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
    }
}
