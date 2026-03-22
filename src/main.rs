use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::{io, sync::Arc, time::Duration};

use crate::app::App;
use crate::docker::docker_client;
use crate::docker::image_tags_for_running_images;
use docker::download_images;

mod app;
mod docker;
mod tui;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(docker_client()?);

    let images = image_tags_for_running_images(Arc::clone(&client)).await?;
    let images_strs: Vec<&str> = images.keys().map(|i| &**i).collect();

    if images_strs.is_empty() {
        println!("No running containers found.");
        return Ok(());
    }

    let mut app = App::new(&images_strs);

    // Calculate needed height:
    // 3 for header
    // images_strs.len() + 2 for the list with borders
    // 5 for errors (if any) - let's assume worst case for now or just images.len() + 5
    let height = (images_strs.len() + 3 + 2 + 5) as u16;

    // Setup terminal with inline viewport
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;

    // Start background downloads
    let (mut rx, join_handles) = download_images(Arc::clone(&client), &images_strs);

    // Run TUI loop
    let res = run_app(&mut terminal, &mut app, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    // Wait for the handles to actually finish
    for handle in join_handles {
        let _ = handle.await;
    }

    println!();

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    rx: &mut tokio::sync::mpsc::Receiver<crate::docker::Status>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| tui::ui(f, app))?;

        if app.completed_images >= app.total_images {
            // Give a small moment to see the 100% state before returning
            tokio::time::sleep(Duration::from_millis(200)).await;
            terminal.draw(|f| tui::ui(f, app))?;
            return Ok(());
        }

        app.tick();

        // Process any pending status updates
        while let Ok(status) = rx.try_recv() {
            app.handle_status(status);
        }

        // Wait for a terminal event
        if crossterm::event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
}
