use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Start background downloads
    let (mut rx, join_handles) = download_images(Arc::clone(&client), &images_strs);

    // Run TUI loop
    let res = run_app(&mut terminal, &mut app, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    // Wait for the handles to actually finish if we didn't exit early,
    // though if we exit via 'q' we might want to just let them drop or abort.
    for handle in join_handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    rx: &mut tokio::sync::mpsc::Receiver<crate::docker::Status>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| tui::ui(f, app))?;

        // Wait for up to 50ms for a terminal event
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    app.quit = true;
                }
            }
        }

        if app.quit {
            return Ok(());
        }

        // Process all pending status updates
        while let Ok(status) = rx.try_recv() {
            app.handle_status(status);
        }

        // If all downloads are complete, we could auto-exit, but let's just 
        // stay open until 'q' is pressed, or maybe exit automatically?
        // Let's auto-exit when all are done for convenience.
        if app.completed_images >= app.total_images {
            // Draw one last time to show 100% completion
            terminal.draw(|f| tui::ui(f, app))?;
            tokio::time::sleep(Duration::from_millis(500)).await;
            return Ok(());
        }
    }
}
