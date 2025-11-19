use anyhow::Result;
use log::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod core;
mod engine;
mod game;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Rusted Battle...");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rusted Battle")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .with_resizable(true)
        .build(&event_loop)?;

    info!("Window created successfully");

    // Main event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Close requested, shutting down...");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                info!("Window resized to {:?}", physical_size);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Clear window (for now just request another frame)
                window.request_redraw();
            }
            Event::AboutToWait => {
                // Request redraw on next frame
                window.request_redraw();
            }
            _ => {}
        }
    }).map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;

    Ok(())
}

